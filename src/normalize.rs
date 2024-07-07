use std::{
    cmp::max,
    cmp::min,
    u64::{MAX, MIN},
};

use ndarray::{Array1, Array3, Axis};

pub fn normalizer_min_max(array: Array3<u64>) -> Array3<f64> {
    let dim = array.shape();

    let mut final_sum = Array3::<f64>::zeros((dim[0], dim[1], dim[2]));
    for z in 0..dim[2] {
        let mut sum = Array3::<f64>::zeros((dim[0], dim[1], dim[2]));
        for x in 0..dim[0] {
            let mut maximum: u64 = MIN;
            let mut minimum: u64 = MAX;
            for y in 0..dim[1] {
                // print!("{:?},", array.get((x, y, z)).unwrap());
                let value = array.get((x, y, z)).unwrap();
                maximum = max(maximum, *value);
                minimum = min(minimum, *value);
            }

            let maximum: f64 = maximum as f64;
            let minimum: f64 = minimum as f64;
            let difference: f64 = maximum - minimum;
            for y in 0..dim[1] {
                let value = array.get((x, y, z)).unwrap();
                let new = sum.get_mut((x, y, z)).unwrap();
                *new = (*value as f64 - minimum) / difference;
            }
        }
        final_sum += &sum;
    }
    return final_sum;
}

pub fn normalizer_sum_one(array: Array3<u64>) -> Array3<f64> {
    let dim = array.shape();

    let mut final_sum = Array3::<f64>::zeros((dim[0], dim[1], dim[2]));
    for z in 0..dim[2] {
        let mut adder = Array3::<f64>::zeros((dim[0], dim[1], dim[2]));
        for x in 0..dim[0] {
            let mut sum: u64 = 0;
            for y in 0..dim[1] {
                sum += *array.get((x, y, z)).unwrap() as u64;
            }
            let sum: f64 = sum as f64;
            for y in 0..dim[1] {
                let value = array.get((x, y, z)).unwrap();
                let new = adder.get_mut((x, y, z)).unwrap();
                *new = *value as f64 / sum;
            }
        }
        final_sum += &adder;
    }
    return final_sum;
}
