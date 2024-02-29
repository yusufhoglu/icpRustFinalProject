use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod,
};

use candid::{CandidType,Decode,Deserialize,Encode};
use ic_stable_structures::memory_manager::{MemoryId,MemoryManager,VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap,Storable};
use std::{borrow::Cow, cell::RefCell};

#[derive(CandidType,Deserialize,Clone)]

struct Event {
    name: String,
    date: String,
    #[serde(default)]//vektörün içini boşaltmak
    participants: Vec<Participant>
}

#[derive(CandidType,Deserialize)]

enum EventError {
    NoSuchEvent,
    JoinError,
    CancelJoinError,
    GetEventsError,
    AlreadyJoined,
    AlreadyExists
}

impl Storable for Event{
    fn to_bytes(&self) -> Cow <[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 100;

// implement
impl BoundedStorable for Event {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
    
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> ? 
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static EVENTS_MAP: RefCell<StableBTreeMap<u64>>, Event, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId:new(1))),
        )
    )
}

#[ic_cdk::update]
fn create_event(name:String, date:String) -> Result<(), EventError>{
    EVENTS_MAP.with(events_map_ref| {
        let mut events_map = events_map_ref.borrow_mut();
        
        //böyle bir etkinlik ismi ve tarihi var mı yok mu
        for(_, event) in events_map.iter(){
            if event.name == name && event.date == date {
                Err(EventError::AlreadyExists)
            }
        }

        // eğer bir etkinlik yoksa, yeni oluştur
        let new_event = Event {
            name,
            date,
            participants: Vec::new(),
        };

        let new_event_id = events_map.len();
        events_map.insert(new_event_id, new_event);
        
        Ok(());
    
    })
}fn create_event

//Etkinlik katılımcıları
#[ic_cdk::update]
fn join_event(event_id: u64, participant_address: String) -> Result<(), EventError>{
    EVENTS_MAP.with(|events_map_ref|{
        let mut events_map = events_map_ref.borrow_mut();
        // event aldık, klonladık, değiştiricez, güncellicez
        if let Some(mut event) = events_map.get(&event_id){
            if event.participants.iter().any(|p| p.address == participant_address){
                Err(EventError:::AlreadyJoined)
            }

            let new_participant = Participant {address: participant_address};
            event.new_participant.push(new_participant);

            events_map.insert(event_id, event);
            Ok(())
        } else {
            Err(EventError::NoSuchEvent)
        }
    })
}

//Katılımcının katılmayı düşündüğü etkinliğe gitmemesi
fn cancel_event(event_id: u64 , participant_address:String) -> Result<(), EventError>{
    EVENTS_MAP.with(|events_map_ref|{
        let mut events_map = events_map_ref.borrow_mut();

        if let Some(mut event) = events_map.get(&event_id){
            if event.participants.iter().any(|p| p.address == participant_address){
                event.participants.remove(participant_address);
                events_map.insert(event_id, event);
            }else{
                Err(EventError::CancelJoinError);
            }
        }else{
            Err(EventError:NoSuchEvent);
        }
    })
}