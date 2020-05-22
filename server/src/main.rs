use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
use warp::{sse::ServerSentEvent, Filter};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is an event stream sender.
    let users = Arc::new(Mutex::new(HashMap::new()));
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    let cors = warp::cors().allow_any_origin();

    // POST /chat -> send message
    let chat_send = warp::path("move")
        .and(warp::post())
        .and(warp::path::param::<usize>())
        .and(warp::body::content_length_limit(500))
        .and(
            warp::body::bytes().and_then(|body: bytes::Bytes| async move {
                serde_json::from_slice::<Location>(&body)
                    .map_err(|_e| warp::reject::custom(BadLocation))
            }),
        )
        .and(users.clone())
        .map(|my_id, location, users| {
            user_move(my_id, location, &users);
            warp::reply::json(&"ok".to_string())
        })
        .with(cors.clone());

    // GET /chat -> messages stream
    let chat_recv = warp::path("sse")
        .and(warp::get())
        .and(users)
        .map(|users| {
            // reply using server-sent events
            let stream = user_connected(users);
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        })
        .with(cors.clone());

    let routes = chat_recv.or(chat_send);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Message variants.
#[derive(Debug, Clone)]
enum Message {
    UserId(usize),
    Location(UserLocation),
    Kickout(Vec<usize>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    x: u32,
    y: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLocation {
    user_id: usize,
    location: Location,
}

pub struct User {
    user_location: UserLocation,
    tx: mpsc::UnboundedSender<Message>,
}

#[derive(Debug)]
struct BadLocation;
impl warp::reject::Reject for BadLocation {}

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `Message`
type Users = Arc<Mutex<HashMap<usize, User>>>;

fn user_connected(
    users: Users,
) -> impl Stream<Item = Result<impl ServerSentEvent + Send + 'static, warp::Error>> + Send + 'static
{
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new user: {}", my_id);

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the event source...
    let (tx, rx) = mpsc::unbounded_channel();

    tx.send(Message::UserId(my_id))
        // rx is right above, so this cannot fail
        .unwrap();

    {
        let mut users = users.lock().unwrap();

        let location = Location { x: 0, y: 0 };

        for user in users.values() {
            tx.send(Message::Location(user.user_location.clone()))
                .unwrap();
        }

        let user_location = UserLocation {
            user_id: my_id,
            location: location.clone(),
        };
        // Save the sender in our list of connected users.
        users.insert(my_id, User { user_location, tx });
    }

    // Convert messages into Server-Sent Events and return resulting stream.
    rx.map(|msg| match msg {
        Message::UserId(my_id) => Ok((warp::sse::event("user"), warp::sse::data(my_id)).boxed()),
        Message::Location(location) => {
            Ok((warp::sse::event("location"), warp::sse::json(location)).boxed())
        }
        Message::Kickout(user_ids) => {
            Ok((warp::sse::event("kickout"), warp::sse::json(user_ids)).boxed())
        }
    })
}

fn user_move(user_id: usize, location: Location, users: &Users) {
    // New message from this user, send it to everyone else (except same uid)...
    //
    // We use `retain` instead of a for loop so that we can reap any user that
    // appears to have disconnected.
    let user_location = UserLocation { user_id, location };
    let mut removed_users = vec![];
    users.lock().unwrap().retain(|&uid, user| {
        if uid == user_id {
            user.user_location = user_location.clone();
        }
        if !user
            .tx
            .send(Message::Location(user_location.clone()))
            .is_ok()
        {
            removed_users.push(uid);
            false
        } else {
            true
        }
    });
    if !removed_users.is_empty() {
        for user in users.lock().unwrap().values() {
            let _ = user.tx.send(Message::Kickout(removed_users.clone()));
        }
    }
}
