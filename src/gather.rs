use ndarray::Array3;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn gather_characters(
    acceptable_types: &[char],
    offset_back: isize,
    offset_front: isize,
    file: File,
) -> Array3<u64> {
    let file_size = &file
        .metadata()
        .expect("Could not open metadata of file")
        .len();
    let reader = BufReader::new(file);

    let mut results: HashMap<(char, i16, char), u64> = HashMap::new();
    let mut buffer: Vec<char> = Vec::new();
    let mut progress_counter: u64 = 0;

    let index_map: HashMap<char, usize> = acceptable_types
        .iter()
        .enumerate()
        .map(|(index, &ch)| (ch, index))
        .collect();

    for line in reader.lines() {
        let line = line.expect("Failed to read a line");
        buffer.extend(line.chars());

        if buffer.len() > 8 {
            progress_counter += 4;
            for (i, &c) in buffer.iter().enumerate().skip(4) {
                progress_counter += 1;
                if acceptable_types.contains(&c) {
                    for offset in (offset_back * -1)..=offset_front {
                        if offset != 0 {
                            let index = i as isize + offset;
                            if index >= 0 && (index as usize) < buffer.len() {
                                let found_char = buffer[index as usize];
                                if acceptable_types.contains(&found_char) {
                                    let entry =
                                        results.entry((c, offset as i16, found_char)).or_insert(0);
                                    *entry += 1;
                                }
                            }
                        }
                    }
                }
            }
            buffer.drain(..buffer.len() - 8);
        }
        print!("\rProgress: {:.2}%", progress_counter / file_size);
        // io::stdout().flush().expect("Could not flush buffer");
    }

    let mut data = Array3::<u64>::zeros((
        acceptable_types.len(),
        acceptable_types.len(),
        offset_back as usize + offset_front as usize + 1,
    ));

    for ((base, offset, search), &count) in &results {
        let point = data.get_mut((
            *index_map.get(base).unwrap(),
            *index_map.get(search).unwrap(),
            *offset as usize + offset_back as usize,
        ));
        //
        match point {
            Some(pointe) => *pointe = count,
            None => println!("Could not find base = {base}, offset = {offset}, search = {search}"),
        }
    }
    return data;
}
