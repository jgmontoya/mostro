use crate::util::{send_cant_do_msg, send_dm};

use anyhow::Result;
use mostro_core::message::{Action, Content, Message};
use mostro_core::user::User;
use nostr::nips::nip59::UnwrappedGift;
use nostr_sdk::prelude::*;
use sqlx::{Pool, Sqlite};
use sqlx_crud::Crud;
use tracing::{error, info};

pub async fn admin_add_solver_action(
    msg: Message,
    event: &UnwrappedGift,
    my_keys: &Keys,
    pool: &Pool<Sqlite>,
) -> Result<()> {
    // Get request id
    let request_id = msg.get_inner_message_kind().request_id;

    let inner_message = msg.get_inner_message_kind();
    let content = if let Some(content) = &inner_message.content {
        content
    } else {
        error!("No pubkey found!");
        return Ok(());
    };
    let npubkey = if let Content::TextMessage(p) = content {
        p
    } else {
        error!("No pubkey found!");
        return Ok(());
    };

    // Check if the pubkey is Mostro
    if event.sender.to_string() != my_keys.public_key().to_string() {
        // We create a Message
        send_cant_do_msg(request_id, None, None, &event.sender).await;
        return Ok(());
    }
    let public_key = PublicKey::from_bech32(npubkey)?.to_hex();
    let user = User::new(public_key, 0, 1, 0, 0);
    // Use CRUD to create user
    match user.create(pool).await {
        Ok(r) => info!("Solver added: {:#?}", r),
        Err(ee) => error!("Error creating solver: {:#?}", ee),
    }
    // We create a Message for admin
    let message = Message::new_dispute(request_id, None, Action::AdminAddSolver, None);
    let message = message.as_json()?;
    // Send the message
    let sender_keys = crate::util::get_keys().unwrap();
    send_dm(&event.sender, sender_keys, message).await?;

    Ok(())
}
