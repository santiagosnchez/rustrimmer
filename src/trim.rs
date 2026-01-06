pub fn trim_record(
    qual: &[u8],
    seq: &[u8],
    qual_thr: u8,
    min_len: usize,
    window: usize,
) -> Option<(Vec<u8>, Vec<u8>)> {
    // Phred+33 assumed
    if qual.is_empty() || seq.is_empty() {
        return None;
    }
    let mut start_idx = 0usize;
    let mut end_idx = qual.len().saturating_sub(1);

    if window <= 1 {
        // existing single-base trimming
        while start_idx <= end_idx {
            let q = qual[start_idx].saturating_sub(33);
            if q >= qual_thr {
                break;
            }
            start_idx += 1;
        }

        while end_idx >= start_idx {
            let q = qual[end_idx].saturating_sub(33);
            if q >= qual_thr {
                break;
            }
            if end_idx == 0 {
                break;
            }
            end_idx = end_idx.saturating_sub(1);
        }
    } else {
        // sliding-window trimming by average quality (fallback to single-base if window > read len)
        let n = qual.len();
        if window > n {
            // fallback to single-base trimming
            while start_idx <= end_idx {
                let q = qual[start_idx].saturating_sub(33);
                if q >= qual_thr {
                    break;
                }
                start_idx += 1;
            }

            while end_idx >= start_idx {
                let q = qual[end_idx].saturating_sub(33);
                if q >= qual_thr {
                    break;
                }
                if end_idx == 0 {
                    break;
                }
                end_idx = end_idx.saturating_sub(1);
            }
        } else {
            // compute integer scores (Phred-33) and prefix sums for fast window sums
            let scores: Vec<u32> = qual.iter().map(|b| b.saturating_sub(33) as u32).collect();
            let mut ps: Vec<u32> = Vec::with_capacity(scores.len() + 1);
            ps.push(0);
            for s in &scores {
                ps.push(ps.last().unwrap() + *s);
            }

            let win = window as usize;
            let thr = qual_thr as u32;

            // find left: first window start i where average >= thr
            let mut found_left: Option<usize> = None;
            for i in 0..=n - win {
                let sum = ps[i + win] - ps[i];
                if sum / (win as u32) >= thr {
                    found_left = Some(i);
                    break;
                }
            }
            if let Some(i) = found_left {
                start_idx = i;
            } else {
                // no qualifying window -> entire read trimmed
                start_idx = n; // will trigger start_idx > end_idx
            }

            // find right: last window end j where average >= thr -> set right = j
            let mut found_right: Option<usize> = None;
            for j in (win - 1)..n {
                let start = j + 1 - win;
                let sum = ps[j + 1] - ps[start];
                if sum / (win as u32) >= thr {
                    found_right = Some(j);
                }
            }
            if let Some(j) = found_right {
                end_idx = j;
            } else {
                // no qualifying window found; keep right at 0 (left>right will handle drop)
                end_idx = 0;
            }
        }
    }

    if start_idx > end_idx {
        return None;
    }
    let trimmed_len = end_idx - start_idx + 1;
    if trimmed_len < min_len {
        return None;
    }

    let seq_slice = &seq[start_idx..=end_idx];
    let qual_slice = &qual[start_idx..=end_idx];
    Some((seq_slice.to_vec(), qual_slice.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::trim_record;

    #[test]
    fn trims_low_ends_and_keeps_middle() {
        // seq: A C G T A C G T
        let seq = b"ACGTACGT";
        // qualities: low (10) at ends, high (40) in middle -> Phred+33
        let qual_vals = [10u8, 10, 40, 40, 40, 40, 10, 10];
        let qual: Vec<u8> = qual_vals.iter().map(|q| q + 33).collect();

        let res = trim_record(&qual, seq, 20, 2, 1);
        assert!(res.is_some());
        let (s, q) = res.unwrap();
        assert_eq!(s, b"GTAC".to_vec());
        assert_eq!(q, vec![40 + 33, 40 + 33, 40 + 33, 40 + 33]);
    }

    #[test]
    fn returns_none_if_too_short_after_trim() {
        let seq = b"ACGT";
        let qual_vals = [10u8, 10, 40, 10];
        let qual: Vec<u8> = qual_vals.iter().map(|q| q + 33).collect();
        // With min_len 3, trimmed middle is length 1 -> should be dropped
        let res = trim_record(&qual, seq, 20, 3, 1);
        assert!(res.is_none());
    }

    #[test]
    fn returns_none_if_all_low() {
        let seq = b"AAAA";
        let qual = vec![10u8 + 33; 4];
        let res = trim_record(&qual, seq, 20, 1, 1);
        assert!(res.is_none());
    }

    #[test]
    fn sliding_window_only_trims_edges_not_center() {
        // Construct a read with low-quality edges and a low-quality center
        // Layout: [low-edge x3][high x4][low-center x3][high x4][low-edge x3] => total 18
        let mut seq = Vec::new();
        seq.extend_from_slice(b"AAAAAAAAA" /* placeholder */);

        // qualities: low=10, high=40 (Phred)
        let mut quals: Vec<u8> = Vec::new();
        quals.extend(vec![10u8 + 33; 3]); // left edge low
        quals.extend(vec![40u8 + 33; 4]); // high
        quals.extend(vec![10u8 + 33; 3]); // low center
        quals.extend(vec![40u8 + 33; 4]); // high
        quals.extend(vec![10u8 + 33; 3]); // right edge low

        // sequence length must match quals
        let seq = vec![b'A'; quals.len()];

        // Window trimming with window=3 and qual_thr=20 should remove only edges
        let res = trim_record(&quals, &seq, 20, 1, 3);
        assert!(res.is_some());
        let (s, q) = res.unwrap();
        // Some trimming should have occurred (edges removed), but exact amount
        // depends on average-window behavior; ensure we trimmed something.
        assert!(s.len() < quals.len());
        // central low-quality block should still be present inside remaining quals
        let remaining_quals: Vec<u8> = q.into_iter().collect();
        // check that there exists a run of three low-quality chars (10+33)
        let low_char = 10u8 + 33;
        let has_center_low = remaining_quals
            .windows(3)
            .any(|w| w == [low_char, low_char, low_char]);
        assert!(
            has_center_low,
            "expected central low-quality region to remain"
        );
    }
}
