use lazy_static::lazy_static;
use rand::{rngs::ThreadRng, Rng};
use std::{
    collections::{HashMap, VecDeque},
    ops::{Add, Div, Index, Mul, MulAssign},
    sync::mpsc,
    sync::{Arc, Mutex},
    thread,
};

use crate::{job, job::JobMessage};
use crate::model::VowelEstimate;

const LIP_SYNC_UPDATED: &str = "lip_sync_updated";
const LIP_SYNC_PANICKED: &str = "lip_sync_panicked";

pub struct LipSync {
    join_handle: Option<thread::JoinHandle<()>>,
    sender: mpsc::Sender<JobMessage>,
    receiver: mpsc::Receiver<JobMessage>,
}

unsafe impl Sync for LipSync {}

unsafe impl Send for LipSync {}

impl LipSync {
    pub fn new() -> Self {
        let (jh, s, r) = job::create_job().expect("Unable to create job thread");

        LipSync {
            join_handle: Some(jh),
            sender: s,
            receiver: r,
        }
    }

    pub fn update(&mut self, stream: Vec<f32>) {
        self.sender
            .send(JobMessage::InputData(stream))
            .expect("Unable to send stream to thread");
    }

    pub fn poll(&self) -> Option<VowelEstimate> {
        match self.receiver.try_recv() {
            Ok(v) => match v {
                JobMessage::OutputData(od) => {
                    // println!("{:?} ", od);
                    return Some(od);
                }
                _ => {
                    // Unexpected data
                    self.sender.send(JobMessage::Shutdown).expect("When shutting down thread because of invalid message, encountered error. Shutting down anyways.");
                    return None;
                }
            },
            Err(e) => {
                if e == mpsc::TryRecvError::Disconnected {
                    println!("LIP_SYNC_PANICKED with error: {}", e);
                    return None;
                }
            }
        }

        return None;
    }

    pub fn shutdown(&mut self) {
        self.sender.send(JobMessage::Shutdown).expect("When shutting down thread because of invalid message, encoutered error. Shutting down anyways.");
        self.join_handle
            .take()
            .expect("Unable to take join_handle")
            .join()
            .expect("Unable to join thread");
    }
}
