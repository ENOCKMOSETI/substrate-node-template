// IPFS implementation
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error, weights::Weight};
use frame_system::{self as system, ensure_signed};
use sp_core::offchain::{Duration, IpfsRequest, IpfsResponse, OpaqueMultiaddr, Timestamp};
use sp_io::offchain::timestamp;
use sp_runtime::offchain::ipfs;
use sp_std::{str, vec::Vec};

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, PartialEq)]
enum ConnectionCommand {
    ConnectTo(OpaqueMultiaddr),
    DisconnectFrom(OpaqueMultiaddr),
}

#[derive(Encode, Decode, PartialEq)]
enum DataCommand {
    AddBytes(Vec<u8>),
    CatBytes(Vec<u8>),
    InsertPin(Vec<u8>),
    RemoveBlock(Vec<u8>),
    RemovePin(Vec<u8>),
}

#[derive(Encode, Decode, PartialEq)]
enum DhtCommand {
    FindPeer(Vec<u8>),
    GetProviders(Vec<u8>),
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        // A list of addresses to connect to and disconnect from.
        pub ConnectionQueue: Vec<ConnectionCommand>;
        // A queue of data to publish or obtain on IPFS.
        pub DataQueue: Vec<DataCommand>;
        // A list of requests to the DHT.
        pub DhtQueue: Vec<DhtCommand>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        ConnectionRequested(AccountId),
        DisconnectRequested(AccountId),
        QueuedDataToAdd(AccountId),
        QueuedDataToCat(AccountId),
        QueuedDataToPin(AccountId),
        QueuedDataToRemove(AccountId),
        QueuedDataToUnpin(AccountId),
        FindPeerIssued(AccountId),
        FindProvidersIssued(AccountId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        CantCreateRequest,
        RequestTimeout,
        RequestFailed,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        // needs to be synchronized with offchain_worker actitivies
        fn on_initialize(block_number: T::BlockNumber) -> Weight {
            ConnectionQueue::kill();
            DhtQueue::kill();

            if block_number % 2.into() == 1.into() {
                DataQueue::kill();
            }

            0
        }

        /// Mark a `Multiaddr` as a desired connection target. The connection will be established
        /// during the next run of the off-chain `connection_housekeeping` process.
        #[weight = 100_000]
        pub fn ipfs_connect(origin, addr: Vec<u8>) {
            let who = ensure_signed(origin)?;
            let cmd = ConnectionCommand::ConnectTo(OpaqueMultiaddr(addr));

            ConnectionQueue::mutate(|cmds| if !cmds.contains(&cmd) { cmds.push(cmd) });
            Self::deposit_event(RawEvent::ConnectionRequested(who));
        }

        /// Queues a `Multiaddr` to be disconnected. The connection will be severed during the next
        /// run of the off-chain `connection_housekeeping` process.
        #[weight = 500_000]
        pub fn ipfs_disconnect(origin, addr: Vec<u8>) {
            let who = ensure_signed(origin)?;
            let cmd = ConnectionCommand::DisconnectFrom(OpaqueMultiaddr(addr));

            ConnectionQueue::mutate(|cmds| if !cmds.contains(&cmd) { cmds.push(cmd) });
            Self::deposit_event(RawEvent::DisconnectRequested(who));
        }

        /// Add arbitrary bytes to the IPFS repository. The registered `Cid` is printed out in the
        /// logs.
        #[weight = 200_000]
        pub fn ipfs_add_bytes(origin, data: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DataQueue::mutate(|queue| queue.push(DataCommand::AddBytes(data)));
            Self::deposit_event(RawEvent::QueuedDataToAdd(who));
        }

        /// Find IPFS data pointed to by the given `Cid`; if it is valid UTF-8, it is printed in the
        /// logs verbatim; otherwise, the decimal representation of the bytes is displayed instead.
        #[weight = 100_000]
        pub fn ipfs_cat_bytes(origin, cid: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DataQueue::mutate(|queue| queue.push(DataCommand::CatBytes(cid)));
            Self::deposit_event(RawEvent::QueuedDataToCat(who));
        }

        /// Add arbitrary bytes to the IPFS repository. The registered `Cid` is printed out in the
        /// logs.
        #[weight = 300_000]
        pub fn ipfs_remove_block(origin, cid: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DataQueue::mutate(|queue| queue.push(DataCommand::RemoveBlock(cid)));
            Self::deposit_event(RawEvent::QueuedDataToRemove(who));
        }

        /// Pins a given `Cid` non-recursively.
        #[weight = 100_000]
        pub fn ipfs_insert_pin(origin, cid: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DataQueue::mutate(|queue| queue.push(DataCommand::InsertPin(cid)));
            Self::deposit_event(RawEvent::QueuedDataToPin(who));
        }

        /// Unpins a given `Cid` non-recursively.
        #[weight = 100_000]
        pub fn ipfs_remove_pin(origin, cid: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DataQueue::mutate(|queue| queue.push(DataCommand::RemovePin(cid)));
            Self::deposit_event(RawEvent::QueuedDataToUnpin(who));
        }

        /// Find addresses associated with the given `PeerId`.
        #[weight = 100_000]
        pub fn ipfs_dht_find_peer(origin, peer_id: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DhtQueue::mutate(|queue| queue.push(DhtCommand::FindPeer(peer_id)));
            Self::deposit_event(RawEvent::FindPeerIssued(who));
        }

        /// Find the list of `PeerId`s known to be hosting the given `Cid`.
        #[weight = 100_000]
        pub fn ipfs_dht_find_providers(origin, cid: Vec<u8>) {
            let who = ensure_signed(origin)?;

            DhtQueue::mutate(|queue| queue.push(DhtCommand::GetProviders(cid)));
            Self::deposit_event(RawEvent::FindProvidersIssued(who));
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            // process connect/disconnect commands
            if let Err(e) = Self::connection_housekeeping() {
                debug::error!("IPFS: Encountered an error during connection housekeeping: {:?}", e);
            }

            // process requests to the DHT
            if let Err(e) = Self::handle_dht_requests() {
                debug::error!("IPFS: Encountered an error while processing DHT requests: {:?}", e);
            }

            // process Ipfs::{add, get} queues every other block
            if block_number % 2.into() == 1.into() {
                if let Err(e) = Self::handle_data_requests() {
                    debug::error!("IPFS: Encountered an error while processing data requests: {:?}", e);
                }
            }

            // display some stats every 5 blocks
            if block_number % 5.into() == 0.into() {
                if let Err(e) = Self::print_metadata() {
                    debug::error!("IPFS: Encountered an error while obtaining metadata: {:?}", e);
                }
            }
        }
    }
}

impl<T: Trait> Module<T> {
    // send a request to the local IPFS node; can only be called be an off-chain worker
    fn ipfs_request(req: IpfsRequest, deadline: impl Into<Option<Timestamp>>) -> Result<IpfsResponse, Error<T>> {
        let ipfs_request = ipfs::PendingRequest::new(req).map_err(|_| Error::<T>::CantCreateRequest)?;
        ipfs_request.try_wait(deadline)
            .map_err(|_| Error::<T>::RequestTimeout)?
            .map(|r| r.response)
            .map_err(|e| {
                if let ipfs::Error::IoError(err) = e {
                    debug::error!("IPFS: request failed: {}", str::from_utf8(&err).unwrap());
                } else {
                    debug::error!("IPFS: request failed: {:?}", e);
                }
                Error::<T>::RequestFailed
            })
    }

    fn connection_housekeeping() -> Result<(), Error<T>> {
        let mut deadline;

        for cmd in ConnectionQueue::get() {
            deadline = Some(timestamp().add(Duration::from_millis(1_000)));

            match cmd {
                // connect to the desired peers if not yet connected
                ConnectionCommand::ConnectTo(addr) => {
                    match Self::ipfs_request(IpfsRequest::Connect(addr.clone()), deadline) {
                        Ok(IpfsResponse::Success) => {
                            debug::info!(
                                "IPFS: connected to {}",
                                str::from_utf8(&addr.0).expect("our own calls can be trusted to be UTF-8; qed")
                            );
                        }
                        Ok(_) => unreachable!("only Success can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: connect error: {:?}", e),
                    }
                }
                // disconnect from peers that are no longer desired
                ConnectionCommand::DisconnectFrom(addr) => {
                    match Self::ipfs_request(IpfsRequest::Disconnect(addr.clone()), deadline) {
                        Ok(IpfsResponse::Success) => {
                            debug::info!(
                                "IPFS: disconnected from {}",
                                str::from_utf8(&addr.0).expect("our own calls can be trusted to be UTF-8; qed")
                            );
                        }
                        Ok(_) => unreachable!("only Success can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: disconnect error: {:?}", e),
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_dht_requests() -> Result<(), Error<T>> {
        let mut deadline;

        for cmd in DhtQueue::get() {
            deadline = Some(timestamp().add(Duration::from_millis(1_000)));

            match cmd {
                // find the known addresses of the given peer
                DhtCommand::FindPeer(peer_id) => {
                    match Self::ipfs_request(IpfsRequest::FindPeer(peer_id.clone()), deadline) {
                        Ok(IpfsResponse::FindPeer(addrs)) => {
                            debug::info!(
                                "IPFS: found the following addresses of {}: {:?}",
                                str::from_utf8(&peer_id).expect("our own calls can be trusted to be UTF-8; qed"),
                                addrs.iter()
                                    .map(|addr| str::from_utf8(&addr.0)
                                        .expect("our node's results can be trusted to be UTF-8; qed"))
                                    .collect::<Vec<_>>()
                            );
                        }
                        Ok(_) => unreachable!("only FindPeer can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: find peer error: {:?}", e),
                    }
                }
                // disconnect from peers that are no longer desired
                DhtCommand::GetProviders(cid) => {
                    match Self::ipfs_request(IpfsRequest::GetProviders(cid.clone()), deadline) {
                        Ok(IpfsResponse::GetProviders(peer_ids)) => {
                            debug::info!(
                                "IPFS: found the following providers of {}: {:?}",
                                str::from_utf8(&cid).expect("our own calls can be trusted to be UTF-8; qed"),
                                peer_ids.iter()
                                    .map(|peer_id| str::from_utf8(&peer_id)
                                        .expect("our node's results can be trusted to be UTF-8; qed"))
                                    .collect::<Vec<_>>()
                            );
                        }
                        Ok(_) => unreachable!("only GetProviders can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: find providers error: {:?}", e),
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_data_requests() -> Result<(), Error<T>> {
        let data_queue = DataQueue::get();
        let len = data_queue.len();
        if len != 0 {
            debug::info!("IPFS: {} entr{} in the data queue", len, if len == 1 { "y" } else { "ies" });
        }

        let deadline = Some(timestamp().add(Duration::from_millis(1_000)));
        for cmd in data_queue.into_iter() {
            match cmd {
                DataCommand::AddBytes(data) => {
                    match Self::ipfs_request(IpfsRequest::AddBytes(data.clone()), deadline) {
                        Ok(IpfsResponse::AddBytes(cid)) => {
                            debug::info!(
                                "IPFS: added data with Cid {}",
                                str::from_utf8(&cid).expect("our own IPFS node can be trusted here; qed")
                            );
                        },
                        Ok(_) => unreachable!("only AddBytes can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: add error: {:?}", e),
                    }
                }
                DataCommand::CatBytes(data) => {
                    match Self::ipfs_request(IpfsRequest::CatBytes(data.clone()), deadline) {
                        Ok(IpfsResponse::CatBytes(data)) => {
                            if let Ok(str) = str::from_utf8(&data) {
                                debug::info!("IPFS: got data: {:?}", str);
                            } else {
                                debug::info!("IPFS: got data: {:x?}", data);
                            };
                        },
                        Ok(_) => unreachable!("only CatBytes can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: error: {:?}", e),
                    }
                }
                DataCommand::RemoveBlock(cid) => {
                    match Self::ipfs_request(IpfsRequest::RemoveBlock(cid), deadline) {
                        Ok(IpfsResponse::RemoveBlock(cid)) => {
                            debug::info!(
                                "IPFS: removed a block with Cid {}",
                                str::from_utf8(&cid).expect("our own IPFS node can be trusted here; qed")
                            );
                        },
                        Ok(_) => unreachable!("only RemoveBlock can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: remove block error: {:?}", e),
                    }
                }
                DataCommand::InsertPin(cid) => {
                    match Self::ipfs_request(IpfsRequest::InsertPin(cid.clone(), false), deadline) {
                        Ok(IpfsResponse::Success) => {
                            debug::info!(
                                "IPFS: pinned data with Cid {}",
                                str::from_utf8(&cid).expect("our own request can be trusted to be UTF-8; qed")
                            );
                        },
                        Ok(_) => unreachable!("only Success can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: insert pin error: {:?}", e),
                    }
                }
                DataCommand::RemovePin(cid) => {
                    match Self::ipfs_request(IpfsRequest::RemovePin(cid.clone(), false), deadline) {
                        Ok(IpfsResponse::Success) => {
                            debug::info!(
                                "IPFS: unpinned data with Cid {}",
                                str::from_utf8(&cid).expect("our own request can be trusted to be UTF-8; qed")
                            );
                        },
                        Ok(_) => unreachable!("only Success can be a response for that request type; qed"),
                        Err(e) => debug::error!("IPFS: remove pin error: {:?}", e),
                    }
                }
            }
        }

        Ok(())
    }

    fn print_metadata() -> Result<(), Error<T>> {
        let deadline = Some(timestamp().add(Duration::from_millis(200)));

        let peers = if let IpfsResponse::Peers(peers) = Self::ipfs_request(IpfsRequest::Peers, deadline)? {
            peers
        } else {
            unreachable!("only Peers can be a response for that request type; qed");
        };
        let peer_count = peers.len();

        debug::info!(
            "IPFS: currently connected to {} peer{}",
            peer_count,
            if peer_count == 1 { "" } else { "s" },
        );

        Ok(())
    }
}