use std::{fs, time::Instant};

use image::io::Reader as ImageReader;
use image::GenericImageView;
use mpi::traits::{Communicator, Destination, Equivalence, Source};

#[repr(C)]
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

    //pub fn to_bytes(&self) -> [u8; 768] {
    //    let mut bytes = [0u32; 256 * 3];
    //    bytes[0..256].clone_from_slice(&self.r);
    //    bytes[256..512].clone_from_slice(&self.g);
    //    bytes[512..768].clone_from_slice(&self.b);
    //    bytes
    //}

    //pub fn from_bytes(bytes: &[u32; 768]) -> Self {
    //    let mut r = [0u32; 256];
    //    let mut g = [0u32; 256];
    //    let mut b = [0u32; 256];

    //    r.clone_from_slice(&bytes[0..256]);
    //    g.clone_from_slice(&bytes[256..512]);
    //    b.clone_from_slice(&bytes[512..]);

    //    Self { r, g, b }
    //}
}

unsafe impl Equivalence for Histogram {
    type Out = mpi::datatype::UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        mpi::datatype::UserDatatype::contiguous(768, &u32::equivalent_datatype())
    }
}

#[repr(C)]
struct Filename {
    name: [u8; 256],
}

unsafe impl Equivalence for Filename {
    type Out = mpi::datatype::UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        mpi::datatype::UserDatatype::contiguous(256, &u8::equivalent_datatype())
    }
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
        for path in paths {
            let p = path.unwrap().path();
            let p_bytes = p.to_str().unwrap().as_bytes();
            let mut msg = [0u8; 256];
            msg[0..p_bytes.len()].clone_from_slice(p_bytes);

            println!("sending {p:?} to {send_to}");
            let filename = Filename { name: msg };
            world.process_at_rank(send_to).send(&filename);
            println!("done sending");

            send_to = (send_to + 1) % size;
            if send_to == 0 {
                send_to += 1;
            }
        }

        for rank in 1..size {
            let msg = [0u8; 256];
            let filename = Filename { name: msg };
            world.process_at_rank(rank).send(&filename);
        }

        let mut hist = Histogram::new();
        for i in 0..3500 {
            let (msg, _status) = world.any_process().receive::<Histogram>();
            println!("image: {i}");
            hist = hist.join(msg);
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
            let (Filename { name: msg }, _status) = world.process_at_rank(0).receive::<Filename>();

            let zero = msg
                .iter()
                .enumerate()
                .find_map(|(i, v)| if *v == 0 { Some(i) } else { None })
                .unwrap();

            if zero == 0 {
                println!("BREAKING");
                break;
            }

            let mut hist = Histogram::new();
            let filename = String::from_utf8_lossy(&msg[0..zero]).to_string();
            println!("{rank}: processing: {filename}");
            let img: image::DynamicImage = ImageReader::open(&filename).unwrap().decode().unwrap();
            for (_, _, pixel) in img.pixels() {
                hist.r[pixel[0] as usize] += 1;
                hist.g[pixel[1] as usize] += 1;
                hist.b[pixel[2] as usize] += 1;
            }

            world.process_at_rank(0).send(&hist);
            println!("{rank}: done processing: {filename}");
        }
    }
}
