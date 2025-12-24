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
    let mut left = 0usize;
    let mut right = qual.len().saturating_sub(1);

    if window <= 1 {
        // existing single-base trimming
        while left <= right {
            let q = qual[left].saturating_sub(33);
            if q >= qual_thr {
                break;
            }
            left += 1;
        }

        while right >= left {
            let q = qual[right].saturating_sub(33);
            if q >= qual_thr {
                break;
            }
            if right == 0 {
                break;
            }
            right = right.saturating_sub(1);
        }
    } else {
        // sliding-window trimming: require a full window of size `window` with all quals >= qual_thr
        // left side: find first index where window [i .. i+window-1] all >= qual_thr
        while left + window - 1 <= right {
            let mut ok = true;
            for j in left..(left + window) {
                let q = qual[j].saturating_sub(33);
                if q < qual_thr {
                    ok = false;
                    break;
                }
            }
            if ok {
                break;
            }
            left += 1;
        }

        // right side: find last index where window [i-window+1 .. i] all >= qual_thr
        while right + 1 >= window && right >= left {
            let start = right.saturating_sub(window - 1);
            let mut ok = true;
            for j in start..=right {
                let q = qual[j].saturating_sub(33);
                if q < qual_thr {
                    ok = false;
                    break;
                }
            }
            if ok {
                break;
            }
            if right == 0 {
                break;
            }
            right = right.saturating_sub(1);
        }
    }

    if left > right {
        return None;
    }
    let trimmed_len = right - left + 1;
    if trimmed_len < min_len {
        return None;
    }

    let seq_slice = &seq[left..=right];
    let qual_slice = &qual[left..=right];
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
        assert_eq!(q, vec![40+33,40+33,40+33,40+33]);
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
}
