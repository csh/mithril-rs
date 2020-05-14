use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::{Rng, SeedableRng};
use tokio_util::codec::{Decoder, Encoder};

pub struct RunescapeCodec {
    encoding_rng: Option<rand_isaac::IsaacRng>,
    decoding_rng: Option<rand_isaac::IsaacRng>,
}

impl RunescapeCodec {
    pub fn new() -> Self {
        RunescapeCodec {
            decoding_rng: None,
            encoding_rng: None,
        }
    }

    pub fn set_isaac_keys(&mut self, server_key: u64, client_key: u64) {
        let seed_inputs = [
            client_key.wrapping_shr(32) as u32,
            client_key as u32,
            server_key.wrapping_shr(32) as u32,
            server_key as u32,
        ];

        self.decoding_rng = Some(rand_isaac::IsaacRng::from_seed(prepare_isaac_seed(
            seed_inputs,
            0,
        )));
        self.encoding_rng = Some(rand_isaac::IsaacRng::from_seed(prepare_isaac_seed(
            seed_inputs,
            50,
        )));
    }
}

fn prepare_isaac_seed(seed_input: [u32; 4], increment: u32) -> [u8; 32] {
    let mut seed = BytesMut::with_capacity(32);
    for input in &seed_input {
        seed.put_u32_le(input + increment);
    }
    seed.put(&[0u8; 16][..]);

    let mut actual_seed = [0u8; 32];
    seed.copy_to_slice(&mut actual_seed);
    actual_seed
}

impl Encoder<Bytes> for RunescapeCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put(item);
        Ok(())
    }
}

impl Decoder for RunescapeCodec {
    type Item = BytesMut;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() > 0 {
            let len = src.len();

            let buf = src.split_to(len);
            if self.decoding_rng.is_some() {
                let isaac = self.decoding_rng.as_mut().unwrap();
                log::debug!("Decrypted packet ID: {}", buf[0] - isaac.gen::<u8>());
            }

            Ok(Some(buf))
        } else {
            Ok(None)
        }
    }
}
