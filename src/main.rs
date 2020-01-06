use shipyard::prelude::*;
use futures_signals::signal_vec::{SignalVec, SignalVecExt, MutableVec, VecDiff};
use lazy_static::lazy_static;
use futures::future::ready;
use futures::executor::block_on;

struct EntityList (pub MutableVec<EntityId>);
struct Label (pub String);

lazy_static! {
    pub static ref WORLD:World = {
        let world = World::new::<(Label, )>();
        world.add_unique(EntityList(MutableVec::new()));
        world
    };
}

fn main() {

    //create the signals *before* adding an entity
    //this way the "bad" one will see the old snapshot
    let sig_good = approach_good();
    let sig_bad = approach_bad();

    //for testing, switch this assignment
    let sig = sig_bad;

    //add the entity
    add_entity();

    //spawn the future to run the signal listener
    block_on(sig.for_each(|change| {
        match change {
            VecDiff::Push { value } => {
                println!("[GOT LABEL] {}", value);
            },
            _ => {}
        }
        ready(())
    }));
}

fn add_entity() {
    let entity = WORLD.run::<(EntitiesMut, &mut Label), _, _>(|(mut entities, mut labels)| {
        entities.add_entity(&mut labels, Label("hello!".to_string()))
    });

    WORLD.run::<Unique<&mut EntityList>, _, _>(|list| {
        list.0.lock_mut().push(entity);
    });
}

// in the good approach, we get the signal first
// then in its map function we run the world again
fn approach_good() -> impl SignalVec<Item = String> {
    let list_signal = WORLD.run::<Unique<&EntityList>, _, _>(|list| {
        list.0.signal_vec()
    });

    list_signal.map(|entity_id| {
        WORLD.run::<&Label, _, _>(|labels| {
            (labels).get(entity_id).unwrap().0.to_string()
        })
    })
}

// in the bad approach we run the World
// and then inside of that create our signal (and the map uses the views from same run)
fn approach_bad() -> impl SignalVec<Item = String> {
    WORLD.run::<(Unique<&EntityList>, &Label), _, _>(|(list, labels)| {
        list.0.signal_vec()
            .map(move |entity_id| {
                (&labels).get(entity_id).unwrap().0.to_string()
            })
    })
}
