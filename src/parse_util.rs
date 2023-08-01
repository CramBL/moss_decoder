use pyo3::{exceptions::PyAssertionError, PyResult};

use crate::moss_protocol::MossWord;

/// Get trailer N's byte index in the given bytes.
#[inline]
pub(super) fn find_trailer_n_idx(bytes: &[u8], n: usize) -> PyResult<usize> {
    let mut last_trailer_idx = 0;
    for i in 0..n {
        if let Some(header_idx) = bytes[last_trailer_idx..]
            .iter()
            .position(|b| MossWord::UNIT_FRAME_HEADER_RANGE.contains(b))
        {
            if let Some(trailer_idx) = &bytes[last_trailer_idx + header_idx..]
                .iter()
                .position(|b| *b == MossWord::UNIT_FRAME_TRAILER)
            {
                last_trailer_idx += header_idx + trailer_idx + 1;
            } else {
                return Err(PyAssertionError::new_err(format!(
                    "No Unit Frame Trailer found for packet {packet_cnt}",
                    packet_cnt = i + 1
                )));
            }
        } else {
            return Err(PyAssertionError::new_err(format!(
                "No Unit Frame Header found for packet {packet_cnt}",
                packet_cnt = i + 1
            )));
        }
    }
    Ok(last_trailer_idx - 1)
}

#[cfg(test)]
mod tests {
    use crate::moss_protocol::MossWord;

    #[test]
    fn test_find_trailer_n_idx_1() {
        let trailer_n = 1;
        let packets = crate::moss_protocol::test_util::fake_multiple_events();

        let trailer_idx = super::find_trailer_n_idx(&packets, trailer_n).unwrap();

        println!("trailer {trailer_n}: {}", trailer_idx);
        assert_eq!(
            MossWord::UNIT_FRAME_TRAILER,
            packets[trailer_idx],
            "Expected trailer {trailer_n} to be at index {trailer_idx}, got: {:X}",
            packets[trailer_idx]
        );
    }

    #[test]
    fn test_find_trailer_n_idx_2() {
        let trailer_n = 2;
        let packets = crate::moss_protocol::test_util::fake_multiple_events();

        let trailer_idx = super::find_trailer_n_idx(&packets, trailer_n).unwrap();

        println!("trailer {trailer_n}: {}", trailer_idx);
        assert_eq!(
            MossWord::UNIT_FRAME_TRAILER,
            packets[trailer_idx],
            "Expected trailer {trailer_n} to be at index {trailer_idx}, got: {:X}",
            packets[trailer_idx]
        );
    }

    #[test]
    fn test_find_trailer_n_idx_3() {
        let trailer_n = 3;
        let packets = crate::moss_protocol::test_util::fake_multiple_events();

        let trailer_idx = super::find_trailer_n_idx(&packets, trailer_n).unwrap();

        println!("trailer {trailer_n}: {}", trailer_idx);
        assert_eq!(
            MossWord::UNIT_FRAME_TRAILER,
            packets[trailer_idx],
            "Expected trailer {trailer_n} to be at index {trailer_idx}, got: {:X}",
            packets[trailer_idx]
        );
    }
    #[test]
    fn test_find_trailer_n_idx_4() {
        let trailer_n = 4;
        let packets = crate::moss_protocol::test_util::fake_multiple_events();

        let trailer_idx = super::find_trailer_n_idx(&packets, trailer_n).unwrap();

        println!("trailer {trailer_n}: {}", trailer_idx);
        assert_eq!(
            MossWord::UNIT_FRAME_TRAILER,
            packets[trailer_idx],
            "Expected trailer {trailer_n} to be at index {trailer_idx}, got: {:X}",
            packets[trailer_idx]
        );
    }

    #[test]
    #[should_panic = "No Unit Frame Header found for packet 5"]
    fn test_find_trailer_n_idx_5() {
        pyo3::prepare_freethreaded_python();
        let trailer_n = 5;
        let packets = crate::moss_protocol::test_util::fake_multiple_events();

        let trailer_idx = super::find_trailer_n_idx(&packets, trailer_n).unwrap();

        println!("trailer {trailer_n}: {}", trailer_idx);
        assert_eq!(
            MossWord::UNIT_FRAME_TRAILER,
            packets[trailer_idx],
            "Expected trailer {trailer_n} to be at index {trailer_idx}, got: {:X}",
            packets[trailer_idx]
        );
    }
}
