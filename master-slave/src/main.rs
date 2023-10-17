use mpi::traits::{Communicator, Destination, Source};

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();

    let size = world.size();
    let rank = world.rank();

    let tasks = 100;

    if rank == 0 {
        // Mestre
        let mut send_to = 0;
        for i in 0..tasks {
            let msg = i;
            world.process_at_rank(send_to).send(&msg);

            send_to = (send_to + 1) % size;
            if send_to == 0 {
                send_to += 1;
            }
        }

        for rank in 1..size {
            let msg = -1;
            world.process_at_rank(rank).send(&msg);
        }

        let mut sum = 0;
        for _ in 0..tasks {
            let (msg, _status) = world.any_process().receive::<i32>();
            sum += msg;
        }
        println!("sum: {sum}");
    } else {
        // Trabalhador
        loop {
            let (msg, _status) = world.process_at_rank(0).receive::<i32>();
            if msg == -1 {
                break;
            }
            let result = msg * 2;
            world.process_at_rank(0).send(&result);
        }
    }
}
