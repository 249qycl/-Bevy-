use rblock::score_server::{Score, ScoreServer};
use rblock::{ScoreRequest, ScoreResponse};
use std::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
#[macro_use]
extern crate lazy_static;

pub mod rblock {
    tonic::include_proto!("rblock");
}
#[derive(Default, Debug)]
pub struct RussiaBlockService {}

lazy_static! {
    static ref SCORES: Mutex<Vec<u32>> = Mutex::new(Vec::new());
}

#[tonic::async_trait]
impl Score for RussiaBlockService {
    async fn query_score(
        &self,
        request: Request<ScoreRequest>,
    ) -> Result<Response<ScoreResponse>, Status> {
        let req = request.into_inner();
        let mut scores = SCORES.lock().unwrap();
        scores.push(req.score);
        scores.sort();
        scores.reverse();
        let rank = scores.iter().position(|&x| x == req.score).unwrap();

        let topk = if scores.len() < req.topk as usize {
            scores.clone()
        } else {
            scores[0..req.topk as usize].to_vec()
        };
        let response = ScoreResponse {
            success: true,
            rank: rank as u32,
            scores: topk,
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8020".parse().unwrap();
    let rb_service = RussiaBlockService::default();
    Server::builder()
        .add_service(ScoreServer::new(rb_service))
        .serve(addr)
        .await?;
    Ok(())
}
