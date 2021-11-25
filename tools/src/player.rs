use anime_telnet::encoding::EncodedPacket;
use cyanotype::SubtitlePacket;

use async_broadcast::Sender;
use futures::stream::{Stream, StreamExt};
use std::collections::VecDeque;
use std::pin::Pin;
use tokio::time::{sleep, Instant};

pub async fn play(
    mut video_stream: Pin<Box<dyn Stream<Item = EncodedPacket> + Send>>,
    mut subtitle_stream: Pin<Box<dyn Stream<Item = SubtitlePacket> + Send>>,
    output: Sender<Vec<u8>>,
) -> Result<(), async_broadcast::SendError<Vec<u8>>> {
    let mut subtitle_buffer = VecDeque::new();
    let play_start = Instant::now();
    loop {
        tokio::select! {
            Some(mut video_frame) = video_stream.next() => {
                video_frame.data.pop();


                subtitle_buffer.retain(|v| {
                    if let SubtitlePacket::SRTEntry(e) = v {
                        e.end >= video_frame.time
                    } else if let SubtitlePacket::SSAEntry(e) = v {
                        e.end.unwrap() >= video_frame.time
                    } else {
                        false
                    }
                });



                if let Some(duration) = video_frame.time.checked_sub(play_start.elapsed()) {
                    sleep(duration).await;
                }

               // clean term; send frame; clear ansi styling
                output.broadcast(b"\x1B[1;1H".to_vec()).await?;
                output.broadcast(video_frame.data).await?;
                output.broadcast(b"\x1B[0m\n".to_vec()).await?;

                // show subtitle if available
                if let Some(SubtitlePacket::SRTEntry(e)) = subtitle_buffer.front() {
                    if e.start <= video_frame.time {
                        output.broadcast(b"\x1B[2K ".to_vec()).await?;
                        output.broadcast(e.text.clone().into_bytes()).await?;
                    }
                } else if let Some(SubtitlePacket::SSAEntry(e)) = subtitle_buffer.front() {
                    if e.start.unwrap() <= video_frame.time {
                        output.broadcast(b"\x1B[2K ".to_vec()).await?;
                        let s = substation::parser::text_line(&e.text).unwrap().1.into_iter().filter_map(|v| {
                            if let substation::TextSection::Text(s) = v {
                                Some(s)
                            } else {
                                None
                            }
                        }).collect::<Vec<String>>().join("").replace("\\N","").into_bytes();
                        output.broadcast(s).await?;
                    }
                } else {
                    output.broadcast(b"\x1B[2K ".to_vec()).await?;
                }
            },
            Some(subtitle_chunk) = subtitle_stream.next() => {
                subtitle_buffer.push_back(subtitle_chunk);
            },
            else => break
        }
    }

    Ok(())
}