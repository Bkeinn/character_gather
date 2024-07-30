use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{mpsc, Arc};
use std::thread;

const CHUNKSIZE: usize = 4096 * 4;

pub fn gather_dataset(
    search_char: char,
    acceptable_types: Vec<char>,
    offset_back: isize,
    offset_front: isize,
    file: File,
    threads: usize,
) -> Vec<Vec<char>> {
    let file_size = file.metadata().expect("Could not read file metadata").len();
    let num_chunks = (file_size as usize + CHUNKSIZE + 1) / CHUNKSIZE;
    let file = Arc::new(file);
    let search_char = Arc::new(search_char);
    let acceptable_types = Arc::new(acceptable_types);
    let (tx, rx) = mpsc::channel();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Could not build thread pool builder");

    let thread_spawner = thread::spawn(move || {
        pool.scope(|s| {
            for i in 0..num_chunks {
                let mut file = Arc::clone(&file);
                let tx = tx.clone();
                let search_char = Arc::clone(&search_char);
                let acceptable_types = Arc::clone(&acceptable_types);
                s.spawn(move |_| {
                    let mut chunk = vec![0; CHUNKSIZE.min(file_size as usize - i * CHUNKSIZE)];

                    file.seek(SeekFrom::Start((i * CHUNKSIZE) as u64)).unwrap();
                    let _amount = file.read(&mut chunk).unwrap();
                    let chunk = chunk.iter().map(|c| *c as char).collect();

                    tx.send(chunk_process(
                        &chunk,
                        &acceptable_types,
                        offset_back,
                        offset_front,
                        &search_char,
                    ))
                    .unwrap();
                });
            }
        });
    });
    let mut final_vec = Vec::new();
    let mut counter: f32 = 0.0;
    let num_chunks = num_chunks as f32;
    for received in rx {
        final_vec.extend(received);
        counter += 1.0;
        print!(
            "At {counter} out of {num_chunks} = {:.1}%\r",
            (counter / num_chunks) * 100.0
        );
    }
    thread_spawner.join().unwrap();
    return final_vec;
}

fn chunk_process(
    buffer: &Vec<char>,
    acceptable: &Arc<Vec<char>>,
    offset_back: isize,
    offset_front: isize,
    search_char: &char,
) -> Vec<Vec<char>> {
    let mut data = Vec::new();

    if buffer.len() > offset_back as usize + offset_front as usize {
        for i in offset_back..buffer.len() as isize - offset_front {
            if &buffer[i as usize] == search_char {
                let context: Vec<char> = (-offset_back..=offset_front)
                    .map(|offset| buffer[(i + offset) as usize])
                    .collect();
                if validate_vec(&context, acceptable) {
                    data.push(context);
                }
            }
        }
    }

    return data;
}

fn validate_vec(vector: &Vec<char>, acceptable: &Arc<Vec<char>>) -> bool {
    for character in vector {
        if !acceptable.contains(character) {
            return false;
        }
    }
    return true;
}
