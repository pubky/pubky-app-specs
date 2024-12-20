use crate::traits::HashId;

const SAMPLE_SIZE: usize = 2 * 1024;

pub struct PubkyAppBlob(pub Vec<u8>);
impl HashId for PubkyAppBlob {
    fn get_id_data(&self) -> String {
        // Get the start and end samples
        let start = &self.0[..SAMPLE_SIZE.min(self.0.len())];
        let end = if self.0.len() > SAMPLE_SIZE {
            &self.0[self.0.len() - SAMPLE_SIZE..]
        } else {
            &[]
        };

        // Combine the samples
        let mut combined = Vec::with_capacity(start.len() + end.len());
        combined.extend_from_slice(start);
        combined.extend_from_slice(end);

        base32::encode(base32::Alphabet::Crockford, &combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::HashId;

    #[test]
    fn test_get_id_data_size_is_smaller_than_sample() {
        let blob = PubkyAppBlob(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let id = blob.get_id_data();
        assert_eq!(id, "041061050R3GG28A");
    }
}
