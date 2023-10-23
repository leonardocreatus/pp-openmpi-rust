use std::{fs, time::Instant};

use image::io::Reader as ImageReader;
use image::GenericImageView;
use mpi::traits::*;

#[derive(Equivalence)]
struct Histogram {
    r: [u32; 256],
    g: [u32; 256],
    b: [u32; 256],
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            r: [0; 256],
            g: [0; 256],
            b: [0; 256],
        }
    }

    pub fn join(self, other: Self) -> Self {
        let Self {
            mut r,
            mut g,
            mut b,
        } = self;
        let Self {
            r: other_r,
            g: other_g,
            b: other_b,
        } = other;

        for (this, other) in r.iter_mut().zip(other_r) {
            *this += other;
        }
        for (this, other) in g.iter_mut().zip(other_g) {
            *this += other;
        }
        for (this, other) in b.iter_mut().zip(other_b) {
            *this += other;
        }

        Self { r, g, b }
    }
}

#[derive(Equivalence)]
struct Filename {
    name: [u8; 256],
}

fn main() {
    let paths = fs::read_dir("../images").unwrap();

    let universe = mpi::initialize().unwrap();
    let world = universe.world();

    let size = world.size();
    let rank = world.rank();

    if rank == 0 {
        // Mestre
        let start = Instant::now();

        let mut send_to = 1;
        let mut hist = Histogram::new();
        for path in paths {
            let p = path.unwrap().path();
            let p_bytes = p.to_str().unwrap().as_bytes();
            let mut msg = [0u8; 256];
            msg[0..p_bytes.len()].clone_from_slice(p_bytes);

            println!("sending {p:?} to {send_to}");
            let filename = Filename { name: msg };
            world.process_at_rank(send_to).send(&filename);
            println!("done sending");

            send_to += 1;
            if send_to == size {
                send_to = 1;
                //    for _ in 1..size {
                //        let (msg, _status) = world.any_process().receive::<Histogram>();
                //        println!("received image from: {}", _status.source_rank());
                //        hist = hist.join(msg);
                //    }
            }
        }

        for rank in 1..size {
            let msg = [0u8; 256];
            let filename = Filename { name: msg };
            world.process_at_rank(rank).send(&filename);
        }

        let end = Instant::now();
        println!("reds: {:?}", hist.r);
        println!("greens: {:?}", hist.g);
        println!("blues: {:?}", hist.b);
        eprintln!(
            "time: {}s",
            end.saturating_duration_since(start).as_secs_f64()
        );
    } else {
        // Trabalhador
        loop {
            println!("{rank}: waiting");
            let (filename, _status) = world.process_at_rank(0).receive::<Filename>();
            println!("{rank}: waiting");
            let Filename { name: msg } = filename;

            let zero = msg
                .iter()
                .enumerate()
                .find_map(|(i, v)| if *v == 0 { Some(i) } else { None })
                .unwrap();

            if zero == 0 {
                println!("BREAKING");
                break;
            }

            //let mut hist = Histogram::new();
            //let filename = String::from_utf8_lossy(&msg[0..zero]).to_string();
            //println!("{rank}: processing: {filename}");
            //let img: image::DynamicImage = ImageReader::open(&filename).unwrap().decode().unwrap();
            //for (_, _, pixel) in img.pixels() {
            //    hist.r[pixel[0] as usize] += 1;
            //    hist.g[pixel[1] as usize] += 1;
            //    hist.b[pixel[2] as usize] += 1;
            //}

            ////world.process_at_rank(0).send(&hist);
            //println!("{rank}: done processing: {filename}");
        }
    }
}
