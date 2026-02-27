use std::time::Instant;

fn resample_linear_into_original(input: &[f32], ratio: f64, output: &mut Vec<f32>) {
    if input.is_empty() {
        return;
    }

    let output_len = (input.len() as f64 * ratio).ceil() as usize;
    output.reserve(output_len);

    let inv_ratio = 1.0 / ratio;

    for i in 0..output_len {
        let src_pos = i as f64 * inv_ratio;
        let src_idx = src_pos as usize;
        let frac = (src_pos - src_idx as f64) as f32;

        let sample = if src_idx + 1 < input.len() {
            input[src_idx] * (1.0 - frac) + input[src_idx + 1] * frac
        } else if src_idx < input.len() {
            input[src_idx]
        } else {
            0.0
        };

        output.push(sample);
    }
}

fn resample_linear_into_optimized(input: &[f32], ratio: f64, output: &mut Vec<f32>) {
    if input.is_empty() {
        return;
    }

    let output_len = (input.len() as f64 * ratio).ceil() as usize;
    output.reserve(output_len);

    let inv_ratio = 1.0 / ratio;

    // We can iterate safely while src_idx + 1 < input.len()
    // src_idx = floor(i * inv_ratio)
    // We want floor(i * inv_ratio) <= input.len() - 2
    // i * inv_ratio < input.len() - 1
    // i < (input.len() - 1) * ratio
    let safe_limit = ((input.len().saturating_sub(1) as f64) * ratio).floor() as usize;
    let safe_limit = safe_limit.min(output_len);

    // Hot loop
    for i in 0..safe_limit {
        let src_pos = i as f64 * inv_ratio;
        // Safety: src_pos is guaranteed to be < input.len() - 1
        let src_idx = src_pos as usize;
        let frac = (src_pos - src_idx as f64) as f32;

        // Safety: src_idx and src_idx + 1 are in bounds by calculation of safe_limit
        let p1 = unsafe { *input.get_unchecked(src_idx) };
        let p2 = unsafe { *input.get_unchecked(src_idx + 1) };

        // Algebraic simplification: p1 * (1 - frac) + p2 * frac = p1 + (p2 - p1) * frac
        let sample = p1 + (p2 - p1) * frac;
        output.push(sample);
    }

    // Tail loop
    for i in safe_limit..output_len {
        let src_pos = i as f64 * inv_ratio;
        let src_idx = src_pos as usize;
        let frac = (src_pos - src_idx as f64) as f32;

        let sample = if src_idx + 1 < input.len() {
            // We can use the safe versions here or the simplified math
             let p1 = input[src_idx];
             let p2 = input[src_idx + 1];
             p1 + (p2 - p1) * frac
        } else if src_idx < input.len() {
            input[src_idx]
        } else {
            0.0
        };

        output.push(sample);
    }
}

fn main() {
    let input_len = 480_000;
    let input: Vec<f32> = (0..input_len).map(|i| (i as f32).sin()).collect();
    let mut output = Vec::new();
    let ratio = 16_000.0 / 48_000.0;
    let iterations = 200;

    // Warmup
    for _ in 0..10 {
        output.clear();
        resample_linear_into_original(&input, ratio, &mut output);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        output.clear();
        resample_linear_into_original(&input, ratio, &mut output);
    }
    let duration_orig = start.elapsed();
    println!("Original: {:.2?}", duration_orig);

    // Warmup
    for _ in 0..10 {
        output.clear();
        resample_linear_into_optimized(&input, ratio, &mut output);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        output.clear();
        resample_linear_into_optimized(&input, ratio, &mut output);
    }
    let duration_opt = start.elapsed();
    println!("Optimized: {:.2?}", duration_opt);

    println!("Improvement: {:.2}%", (1.0 - duration_opt.as_secs_f64() / duration_orig.as_secs_f64()) * 100.0);

    // Verify correctness
    output.clear();
    resample_linear_into_original(&input, ratio, &mut output);
    let output_orig = output.clone();

    output.clear();
    resample_linear_into_optimized(&input, ratio, &mut output);
    let output_opt = output.clone();

    if output_orig.len() != output_opt.len() {
        println!("Length mismatch! Orig: {}, Opt: {}", output_orig.len(), output_opt.len());
    } else {
        let mut max_diff = 0.0;
        for (i, (a, b)) in output_orig.iter().zip(output_opt.iter()).enumerate() {
            let diff = (a - b).abs();
            if diff > max_diff {
                max_diff = diff;
            }
        }
        println!("Max diff: {}", max_diff);
        if max_diff < 1e-5 {
            println!("Verification passed!");
        } else {
            println!("Verification failed!");
        }
    }
}
