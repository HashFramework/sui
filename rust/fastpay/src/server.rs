// Copyright (c) Facebook Inc.
// SPDX-License-Identifier: Apache-2.0

#![deny(warnings)]

extern crate env_logger;
extern crate fastpay;
extern crate fastpay_core;

use fastpay::{config::*, network, transport};
use fastpay_core::{authority::*, base_types::*, committee::Committee};

use futures::future::join_all;
use log::*;
use structopt::StructOpt;
use tokio::runtime::Runtime;

#[allow(clippy::too_many_arguments)]
fn make_shard_server(
    local_ip_addr: &str,
    server_config_path: &str,
    committee_config_path: &str,
    initial_accounts_config_path: &str,
    initial_balance: Balance,
    buffer_size: usize,
    cross_shard_queue_size: usize,
    shard: u32,
) -> network::Server {
    let server_config =
        AuthorityServerConfig::read(server_config_path).expect("Fail to read server config");
    let committee_config =
        CommitteeConfig::read(committee_config_path).expect("Fail to read committee config");
    let initial_accounts_config = InitialStateConfig::read(initial_accounts_config_path)
        .expect("Fail to read initial account config");

    let committee = Committee::new(committee_config.voting_rights());
    let num_shards = server_config.authority.num_shards;

    let mut state = AuthorityState::new_shard(
        committee,
        server_config.authority.address,
        server_config.key.copy(),
        shard,
        num_shards,
    );

    // Load initial states
    for address in &initial_accounts_config.addresses {
        if AuthorityState::get_shard(num_shards, address) != shard {
            continue;
        }
        let client = AccountOffchainState {
            balance: initial_balance,
            next_sequence_number: SequenceNumber::from(0),
            pending_confirmation: None,
            confirmed_log: Vec::new(),
            synchronization_log: Vec::new(),
            received_log: Vec::new(),
        };
        state.accounts.insert(*address, client);
    }

    network::Server::new(
        server_config.authority.network_protocol,
        local_ip_addr.to_string(),
        server_config.authority.base_port,
        state,
        buffer_size,
        cross_shard_queue_size,
    )
}

fn make_servers(
    local_ip_addr: &str,
    server_config_path: &str,
    committee_config_path: &str,
    initial_accounts_config_path: &str,
    initial_balance: Balance,
    buffer_size: usize,
    cross_shard_queue_size: usize,
) -> Vec<network::Server> {
    let server_config =
        AuthorityServerConfig::read(server_config_path).expect("Fail to read server config");
    let num_shards = server_config.authority.num_shards;

    let mut servers = Vec::new();
    for shard in 0..num_shards {
        servers.push(make_shard_server(
            local_ip_addr,
            server_config_path,
            committee_config_path,
            initial_accounts_config_path,
            initial_balance,
            buffer_size,
            cross_shard_queue_size,
            shard,
        ))
    }
    servers
}

#[derive(StructOpt)]
#[structopt(
    name = "FastPay Server",
    about = "A byzantine fault tolerant payments sidechain with low-latency finality and high throughput"
)]
struct ServerOpt {
    /// Path to the file containing the server configuration of this FastPay authority (including its secret key)
    #[structopt(long = "server")]
    server: String,

    /// Subcommands. Acceptable values are run and generate.
    #[structopt(subcommand)]
    cmd: ServerCommands,
}

#[derive(StructOpt)]
enum ServerCommands {
    /// Runs a service for each shard of the FastPay authority")
    #[structopt(name = "run")]
    Run {
        /// Maximum size of datagrams received and sent (bytes)
        #[structopt(long = "buffer_size", default_value = transport::DEFAULT_MAX_DATAGRAM_SIZE)]
        buffer_size: String,

        /// Number of cross shards messages allowed before blocking the main server loop
        #[structopt(long = "cross_shard_queue_size", default_value = "1000")]
        cross_shard_queue_size: usize,

        /// Path to the file containing the public description of all authorities in this FastPay committee
        #[structopt(long = "committee")]
        committee: String,

        /// Path to the file describing the initial user accounts
        #[structopt(long = "initial_accounts")]
        initial_accounts: String,

        /// Path to the file describing the initial balance of user accounts
        #[structopt(long = "initial_balance")]
        initial_balance: String,

        /// Runs a specific shard (from 0 to shards-1)
        #[structopt(long = "shard")]
        shard: String,
    },

    /// Generate a new server configuration and output its public description
    #[structopt(name = "generate")]
    Generate {
        /// Chooses a network protocol between Udp and Tcp
        #[structopt(long = "protocol", default_value = "Udp")]
        protocol: String,

        /// Sets the public name of the host
        #[structopt(long = "host")]
        host: String,

        /// Sets the base port, i.e. the port on which the server listens for the first shard
        #[structopt(long = "port")]
        port: u32,

        /// Number of shards for this authority
        #[structopt(long = "shards")]
        shards: u32,
    },
}

fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let matches = ServerOpt::from_args();

    let server_config_path = &matches.server;

    match matches.cmd {
        ServerCommands::Run {
            buffer_size,
            cross_shard_queue_size,
            committee,
            initial_accounts,
            initial_balance,
            shard,
        } => {
            let committee_config_path = &committee;
            let initial_accounts_config_path = &initial_accounts;
            let initial_balance = Balance::from(initial_balance.parse::<i128>().unwrap());
            let buffer_size = buffer_size.parse::<usize>().unwrap();
            // let parsed_cross_shard_queue_size = cross_shard_queue_size.parse::<usize>().unwrap();
            let specific_shard = Some(shard.parse::<u32>().unwrap());

            // Run the server
            let servers = if let Some(shard) = specific_shard {
                info!("Running shard number {}", shard);
                let server = make_shard_server(
                    "0.0.0.0", // Allow local IP address to be different from the public one.
                    server_config_path,
                    committee_config_path,
                    initial_accounts_config_path,
                    initial_balance,
                    buffer_size,
                    cross_shard_queue_size,
                    shard,
                );
                vec![server]
            } else {
                info!("Running all shards");
                make_servers(
                    "0.0.0.0", // Allow local IP address to be different from the public one.
                    server_config_path,
                    committee_config_path,
                    initial_accounts_config_path,
                    initial_balance,
                    buffer_size,
                    cross_shard_queue_size,
                )
            };

            let mut rt = Runtime::new().unwrap();
            let mut handles = Vec::new();
            for server in servers {
                handles.push(async move {
                    let spawned_server = match server.spawn().await {
                        Ok(server) => server,
                        Err(err) => {
                            error!("Failed to start server: {}", err);
                            return;
                        }
                    };
                    if let Err(err) = spawned_server.join().await {
                        error!("Server ended with an error: {}", err);
                    }
                });
            }
            rt.block_on(join_all(handles));
        }

        ServerCommands::Generate {
            protocol,
            host,
            port,
            shards,
        } => {
            let network_protocol = protocol.parse().unwrap();
            let (address, key) = get_key_pair();
            let authority = AuthorityConfig {
                network_protocol,
                address,
                host,
                base_port: port,
                num_shards: shards,
            };
            let server = AuthorityServerConfig { authority, key };
            server
                .write(server_config_path)
                .expect("Unable to write server config file");
            info!("Wrote server config file");
            server.authority.print();
        }
    }
}
