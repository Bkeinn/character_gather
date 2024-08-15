use std::fs::File;
use std::io::Write;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{mpsc, Arc};
use std::thread;

const CHUNKSIZE: usize = 4096 * 4;

pub fn gather_dataset(
    search_char: char,
    acceptable_types: Arc<Vec<char>>,
    offset_back: isize,
    offset_front: isize,
    mut file: File,
    mut result_file: File,
)
// -> Vec<Vec<char>>
{
    let file_size = file.metadata().expect("Could not read file metadata").len();
    let num_chunks = (file_size as usize + CHUNKSIZE + 1) / CHUNKSIZE;
    // let file = Arc::new(file);
    // let search_char = Arc::new(search_char);
    // let acceptable_types = Arc::new(acceptable_types);
    let (tx, rx) = mpsc::channel();

    let workthread = thread::spawn(move || {
        for i in 0..num_chunks {
            // let mut file = Arc::clone(&file);
            let tx = tx.clone();
            // let search_char = Arc::clone(&search_char);
            // let acceptable_types = Arc::clone(&acceptable_types);
            let mut chunk = vec![0; CHUNKSIZE.min(file_size as usize - i * CHUNKSIZE)];

            file.seek(SeekFrom::Start((i * CHUNKSIZE) as u64)).unwrap();
            let _amount = file
                .read(&mut chunk)
                .expect(&format!("Could not open file {:?}", file));
            let chunk = chunk.iter().map(|c| *c as char).collect();

            tx.send(chunk_process(
                &chunk,
                &acceptable_types,
                offset_back,
                offset_front,
                &search_char,
            ))
            .unwrap();
        }
    });

    let mut counter: f32 = 0.0;
    let num_chunks = num_chunks as f32;
    for received in rx {
        append_to_csv(&mut result_file, received).expect("Could not write data to csv");
        counter += 1.0;
        print!(
            "At {counter} out of {num_chunks} = {:.1}%\r",
            (counter / num_chunks) * 100.0
        );
    }
    workthread.join().unwrap();
}

fn append_to_csv(file: &mut File, data: Vec<Vec<char>>) -> std::io::Result<()> {
    // Convert Vec<Vec<char>> to a string where each inner Vec is a comma-separated line
    let mut content = String::new();
    for row in data {
        let line: String = row
            .iter()
            .map(|&c| c.to_string()) // Convert each char to a String
            .collect::<Vec<_>>()
            .join(","); // Join the Vec<String> with commas
        content.push_str(&line);
        content.push_str(",\n"); // Adds a comma at the end of each line
    }
    // Write the content to the file
    file.write_all(content.as_bytes())?;

    Ok(())
}

fn chunk_process(
    buffer: &Vec<char>,
    acceptable: &Vec<char>,
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

fn validate_vec(vector: &Vec<char>, acceptable: &Vec<char>) -> bool {
    for character in vector {
        if !acceptable.contains(character) {
            return false;
        }
    }
    return true;
}
