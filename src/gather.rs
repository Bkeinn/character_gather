use ndarray::{Array3, ArrayBase, Dim, OwnedRepr};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{mpsc, Arc};
use std::{array, thread};

const BUFFERMAX: usize = 4096 * 4;
const CHUNKSIZE: usize = 4096;

pub fn gather_characters(
    acceptable_types: Vec<char>,
    offset_back: isize,
    offset_front: isize,
    file: File,
) -> Array3<u64> {
    let file_size = file.metadata().expect("Could not read file metadata").len();
    let num_chunks = (file_size as usize + CHUNKSIZE + 1) / CHUNKSIZE;
    let file = Arc::new(file);

    let index_map: HashMap<char, usize> = acceptable_types
        .iter()
        .enumerate()
        .map(|(index, &ch)| (ch, index))
        .collect();
    let index_map = Arc::new(index_map);
    let acceptable_types = Arc::new(acceptable_types);
    let (tx, rx) = mpsc::channel();

    let mut final_sum = Array3::<u64>::zeros((
        acceptable_types.len(),
        acceptable_types.len(),
        offset_back as usize + offset_front as usize + 1,
    ));

    for i in 0..num_chunks {
        let mut file = Arc::clone(&file);

        let tx = tx.clone();

        let index_map = Arc::clone(&index_map);
        let acceptable_types = Arc::clone(&acceptable_types);
        thread::spawn(move || {
            let mut chunk = vec![0; CHUNKSIZE.min(file_size as usize - i * CHUNKSIZE)];

            file.seek(SeekFrom::Start((i * CHUNKSIZE) as u64)).unwrap();
            let _amount = file.read(&mut chunk).unwrap();
            let chunk = chunk.iter().map(|c| *c as char).collect();

            tx.send(line_process(
                &chunk,
                &acceptable_types,
                offset_back,
                offset_front,
                &index_map,
            ))
        });
    }

    drop(tx);

    for received in rx {
        final_sum += &received;
    }
    return final_sum;
}

fn line_process(
    buffer: &Vec<char>,
    acceptable: &Arc<Vec<char>>,
    offset_back: isize,
    offset_front: isize,
    index_map: &Arc<HashMap<char, usize>>,
) -> ArrayBase<OwnedRepr<u64>, Dim<[usize; 3]>> {
    let mut data = Array3::<u64>::zeros((
        acceptable.len(),
        acceptable.len(),
        offset_back as usize + offset_front as usize + 1,
    ));
    // eprint!("Buffer:\n{:?}", buffer);
    // eprint!("Acceptable:\n{:?}", acceptable);

    if buffer.len() > offset_back as usize + offset_front as usize {
        for (counter, &character) in buffer.iter().enumerate().skip(offset_back as usize) {
            if acceptable.contains(&character) {
                for offset in -offset_back..=offset_front {
                    if offset != 0 {
                        let suround_char_index = counter as isize + offset;
                        if suround_char_index >= 0 && (suround_char_index as usize) < buffer.len() {
                            let found_char = buffer[suround_char_index as usize];
                            if acceptable.contains(&found_char) {
                                data.get_mut((
                                    *index_map.get(&character).unwrap(),
                                    *index_map.get(&found_char).unwrap(),
                                    offset as usize + offset_back as usize,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    //println!("Thread finished with:\n{:#?}", &data);
    return data;
}
