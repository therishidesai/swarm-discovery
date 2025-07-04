use crate::{
    receiver::{receiver, ReceiverError},
    sender::{self},
    socket::Sockets,
    updater::updater,
    Discoverer,
};
use acto::{AcTokioRuntime, ActoCell, ActoInput};
use hickory_proto::rr::Name;
use std::{mem::replace, net::IpAddr};

pub enum Input {
    RemoveAll,
    RemovePort(u16),
    RemoveAddr(IpAddr),
    AddAddr(u16, Vec<IpAddr>),
    SetTxt(String, Option<String>),
    RemoveTxt(String),
}

pub async fn guardian(
    mut ctx: ActoCell<Input, AcTokioRuntime, Result<(), ReceiverError>>,
    mut discoverer: Discoverer,
    sockets_vec: Vec<Sockets>,
    service_name: Name,
) {
    let callback = replace(&mut discoverer.callback, Box::new(|_, _| {}));
    let tau = discoverer.tau;
    let phi = discoverer.phi;
    let upd_ref = ctx.supervise(
        ctx.spawn("updater", move |ctx| updater(ctx, tau, phi, callback))
            .map_handle(Ok),
    );

    let sn = service_name.clone();
    let sockets_vec_clone = sockets_vec.clone();
    let snd_ref = ctx.supervise(
        ctx.spawn("sender", move |ctx| {
            sender::sender(ctx, sockets_vec_clone, upd_ref, discoverer, sn)
        })
        .map_handle(Ok),
    );

    // Spawn receivers for all sockets
    for (socket_idx, sockets) in sockets_vec.iter().enumerate() {
        if let Some(v4) = sockets.v4() {
            let service_name = service_name.clone();
            let snd_ref = snd_ref.clone();
            let receiver_name = format!("receiver_v4_{}", socket_idx);
            ctx.spawn_supervised(&receiver_name, move |ctx| {
                receiver(ctx, service_name, v4, snd_ref)
            });
        }

        if let Some(v6) = sockets.v6() {
            let service_name = service_name.clone();
            let snd_ref = snd_ref.clone();
            let receiver_name = format!("receiver_v6_{}", socket_idx);
            ctx.spawn_supervised(&receiver_name, move |ctx| {
                receiver(ctx, service_name, v6, snd_ref)
            });
        }
    }

    // only stop when a supervised actor stops
    loop {
        let msg = ctx.recv().await;
        match msg {
            ActoInput::NoMoreSenders => {}
            ActoInput::Supervision { id, name, result } => {
                match result {
                    Ok(Ok(_)) => tracing::warn!("actor {:?} ({}) stopped", id, name),
                    Ok(Err(e)) => {
                        tracing::warn!("actor {:?} ({}) failed: {}", id, name, e)
                    }
                    Err(e) => {
                        tracing::warn!("actor {:?} ({}) aborted: {}", id, name, e);
                    }
                }
                break;
            }
            ActoInput::Message(msg) => {
                snd_ref.send(sender::MdnsMsg::Update(msg));
            }
        }
    }
}
