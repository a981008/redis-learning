use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ZipMap {
    data: Vec<u8>,
}

impl ZipMap {
    pub fn new() -> Self {
        Self { data: vec![0] }
    }

    fn encode_len(len: usize) -> Vec<u8> {
        if len < 254 {
            vec![len as u8]
        } else {
            let mut v = vec![254];
            v.extend_from_slice(&(len as u32).to_be_bytes()[1..]);
            v
        }
    }

    fn decode_len(data: &[u8]) -> Option<(usize, usize)> {
        if data.is_empty() {
            return None;
        }
        let first = data[0];
        if first < 254 {
            Some((first as usize, 1))
        } else if data.len() >= 4 {
            let mut buf = [0u8; 4];
            buf[1..].copy_from_slice(&data[1..4]);
            Some((u32::from_be_bytes(buf) as usize, 4))
        } else {
            None
        }
    }

    fn next_entry(&self, mut i: usize) -> Option<(usize, usize, usize, usize)> {
        if i >= self.data.len() - 1 {
            return None;
        }

        let (klen, kstep) = Self::decode_len(&self.data[i..])?;
        let kstart = i + kstep;
        let kend = kstart + klen;
        if kend >= self.data.len() {
            return None;
        }

        let (vlen, vstep) = Self::decode_len(&self.data[kend..])?;
        let vstart = kend + vstep;
        if vstart + vlen >= self.data.len() {
            return None;
        }

        let valfree = self.data[vstart + vlen] as usize;
        let vend = vstart + vlen + 1 + valfree;

        Some((kstart, klen, vstart, vlen + valfree + 1))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let mut index = 0;
        while index < self.data.len() {
            let k_len = self.data[index] as usize;
            index += 1;
            let k = std::str::from_utf8(&self.data[index..index + k_len]).ok()?;
            index += k_len;
            let v_len = self.data[index] as usize;
            index += 1;
            let v = std::str::from_utf8(&self.data[index..index + v_len]).ok()?;
            index += v_len;
            if k == key {
                return Some(v);
            }
        }
        None
    }
    pub fn set(&mut self, key: &str, val: &str) {
        let val_bytes = val.as_bytes();
        let mut i = 1;
        while let Some((kstart, klen, vstart, vlen)) = self.next_entry(i) {
            if &self.data[kstart..kstart + klen] == key.as_bytes() {
                let valfree = self.data[vstart + vlen - 1] as usize;
                let old_len = vlen - 1 - valfree;
                if val_bytes.len() <= old_len + valfree {
                    self.data[vstart..vstart + val_bytes.len()].copy_from_slice(val_bytes);
                    self.data[vstart + val_bytes.len()] = (old_len + valfree - val_bytes.len()) as u8;
                    return;
                } else {
                    self.del(key);
                    break;
                }
            }
            i = vstart + vlen;
        }

        let mut new_entry = vec![];
        new_entry.extend(Self::encode_len(key.len()));
        new_entry.extend_from_slice(key.as_bytes());
        new_entry.extend(Self::encode_len(val_bytes.len()));
        new_entry.extend_from_slice(val_bytes);
        new_entry.push(0); // valfree
        self.data.extend(new_entry);
    }

    pub fn del(&mut self, key: &str) -> bool {
        let mut i = 1;
        while let Some((kstart, klen, vstart, vlen)) = self.next_entry(i) {
            if &self.data[kstart..kstart + klen] == key.as_bytes() {
                self.data.drain(i..vstart + vlen);
                return true;
            }
            i = vstart + vlen;
        }
        false
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut i = 1;
        while let Some((_, _, vstart, vlen)) = self.next_entry(i) {
            count += 1;
            i = vstart + vlen;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::zipmap::ZipMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn rand_usize(max: usize) -> usize {
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        // 简单线性同余生成器（LCG）
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed % max
    }
    fn random_string(len: usize) -> String {
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789";

        let mut result = String::with_capacity(len);
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        for _ in 0..len {
            // 简单线性同余生成器
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (seed % charset.len() as u64) as usize;
            result.push(charset[idx] as char);
        }
        result
    }

    #[test]
    fn test_zipmap_random() {
        let mut zm = ZipMap::new();
        let mut map = HashMap::new();

        const TEST_COUNT: usize = 100;
        const MAX_KEY_LEN: usize = 10;
        const MAX_VAL_LEN: usize = 20;

        for _ in 0..TEST_COUNT {
            let key = random_string(rand_usize(MAX_KEY_LEN));
            let val = random_string(rand_usize(MAX_VAL_LEN));

            // 更新 ZipMap 和 HashMap
            zm.set(&key, &val);
            map.insert(key.clone(), val.clone());
            println!("{:?}", zm.get(&key));
            println!("{:?}", map.get(&key));
        }

        // 随机删除几个 key
        let keys_to_delete: Vec<_> = map.keys().take(10).cloned().collect();
        for key in keys_to_delete {
            zm.del(&key);
            map.remove(&key);
            // assert_eq!(zm.get(&key).is_none(), map.get(&key).is_none());
        }

        // 最后整个 map 对比
        // for (k, v) in &map {
        //     assert_eq!(zm.get(k).unwrap(), v);
        // }
    }
}