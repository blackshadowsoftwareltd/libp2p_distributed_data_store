use crate::{
    config::{swarm_config, CustomBehaviour, CustomBehaviourEvent},
    input::handle_input_line,
};
use futures::StreamExt;
use libp2p::{
    kad::{self, store::MemoryStore},
    mdns,
    swarm::SwarmEvent,
    Swarm,
};
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};

pub type SwarmBehaviour = Swarm<CustomBehaviour>;
pub type KadMemoryBehav = kad::Behaviour<MemoryStore>;

pub async fn run() {
    if let Ok(mut swarm) = swarm_config() {
        let mut stdin = io::BufReader::new(io::stdin()).lines();
        loop {
            select! {
                    Ok(Some(line)) = stdin.next_line() => {
                        handle_input_line(&mut swarm.behaviour_mut().kademlia, line);
                    }

                event = swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening in {address:?}");
                    },
                    SwarmEvent::Behaviour(CustomBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            println!("Discovered {:?}",peer_id);
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                        }
                    }
                    SwarmEvent::Behaviour(CustomBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, ..})) => {
                        match result {
                            kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { key, providers, .. })) => {
                                for peer in providers {
                                    println!(
                                        "Peer {peer:?} provides key {:?}",
                                        std::str::from_utf8(key.as_ref()).unwrap()
                                    );
                                }
                            }
                            kad::QueryResult::GetProviders(Err(err)) => {
                                eprintln!("Failed to get providers: {err:?}");
                            }
                            kad::QueryResult::GetRecord(Ok(
                                kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                                    record: kad::Record { key, value, .. },
                                    ..
                                })
                            )) => {
                                println!(
                                    "Got record {:?} {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap(),
                                    std::str::from_utf8(&value).unwrap(),
                                );
                            }
                            kad::QueryResult::GetRecord(Ok(_)) => {}
                            kad::QueryResult::GetRecord(Err(err)) => {
                                eprintln!("Failed to get record: {err:?}");
                            }
                            kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                                println!(
                                    "Successfully put record {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap()
                                );
                            }
                            kad::QueryResult::PutRecord(Err(err)) => {
                                eprintln!("Failed to put record: {err:?}");
                            }
                            kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                                println!(
                                    "Successfully put provider record {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap()
                                );
                            }
                            kad::QueryResult::StartProviding(Err(err)) => {
                                eprintln!("Failed to put provider record: {err:?}");
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

            }
        }
    }
}
