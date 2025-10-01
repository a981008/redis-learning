pub struct ZipMap {
    buf: Vec<u8>,
    iter_pos: usize,
}

impl ZipMap {
    pub fn new() -> Self {
        ZipMap {
            buf: Vec::new(),
            iter_pos: 0,
        }
    }

    /// 插入或更新 key
    pub fn set(&mut self, key: &str, val: &str) {
        if let Some((pos, klen, vlen)) = self.find_entry(key) {
            // 更新：覆盖原值（仅当长度相等，否则先删除再追加）
            if vlen == val.len() {
                let start = pos + 1 + klen + 1;
                self.buf[start..start + vlen].copy_from_slice(val.as_bytes());
                return;
            } else {
                self.del(key);
            }
        }

        // 追加新 entry: [klen][kbytes][vlen][vbytes]
        self.buf.push(key.len() as u8);
        self.buf.extend_from_slice(key.as_bytes());
        self.buf.push(val.len() as u8);
        self.buf.extend_from_slice(val.as_bytes());
    }

    /// 查询
    pub fn get(&self, key: &str) -> Option<&str> {
        let mut i = 0;
        while i < self.buf.len() {
            let klen = self.buf[i] as usize;
            let kstart = i + 1;
            let kend = kstart + klen;
            let vlen_pos = kend;
            let vlen = self.buf[vlen_pos] as usize;
            let vstart = vlen_pos + 1;
            let vend = vstart + vlen;

            if &self.buf[kstart..kend] == key.as_bytes() {
                return std::str::from_utf8(&self.buf[vstart..vend]).ok();
            }

            i = vend;
        }
        None
    }

    /// 删除
    pub fn del(&mut self, key: &str) -> bool {
        let mut i = 0;
        while i < self.buf.len() {
            let klen = self.buf[i] as usize;
            let kstart = i + 1;
            let kend = kstart + klen;
            let vlen_pos = kend;
            let vlen = self.buf[vlen_pos] as usize;
            let vstart = vlen_pos + 1;
            let vend = vstart + vlen;

            if &self.buf[kstart..kend] == key.as_bytes() {
                self.buf.drain(i..vend);
                return true;
            }

            i = vend;
        }
        false
    }

    /// 内部函数：查找 entry
    fn find_entry(&self, key: &str) -> Option<(usize, usize, usize)> {
        let mut i = 0;
        while i < self.buf.len() {
            let klen = self.buf[i] as usize;
            let kstart = i + 1;
            let kend = kstart + klen;
            let vlen_pos = kend;
            let vlen = self.buf[vlen_pos] as usize;
            let vstart = vlen_pos + 1;
            let vend = vstart + vlen;

            if &self.buf[kstart..kend] == key.as_bytes() {
                return Some((i, klen, vlen));
            }

            i = vend;
        }
        None
    }

    pub fn exists(&self, key: &str) -> bool {
        self.find_entry(key).is_some()
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < self.buf.len() {
            let klen = self.buf[i] as usize;
            let vlen = self.buf[i + 1 + klen] as usize;
            i += 1 + klen + 1 + vlen;
            count += 1;
        }
        count
    }

    pub fn repr(&self) -> String {
        let mut s = String::from("{ ");
        let mut i = 0;
        while i < self.buf.len() {
            let klen = self.buf[i] as usize;
            let kstart = i + 1;
            let kend = kstart + klen;
            let vlen = self.buf[kend] as usize;
            let vstart = kend + 1;
            let vend = vstart + vlen;

            let key = std::str::from_utf8(&self.buf[kstart..kend]).unwrap();
            let val = std::str::from_utf8(&self.buf[vstart..vend]).unwrap();

            s.push_str(&format!("{}: {}, ", key, val));

            i = vend;
        }
        s.push_str("}");
        s
    }

    /// 遍历接口
    pub fn rewind(&mut self) {
        self.iter_pos = 0;
    }

    pub fn next(&mut self) -> Option<(&str, &str)> {
        if self.iter_pos >= self.buf.len() {
            return None;
        }

        let i = self.iter_pos;
        let klen = self.buf[i] as usize;
        let kstart = i + 1;
        let kend = kstart + klen;
        let vlen = self.buf[kend] as usize;
        let vstart = kend + 1;
        let vend = vstart + vlen;

        self.iter_pos = vend;

        let key = std::str::from_utf8(&self.buf[kstart..kend]).unwrap();
        let val = std::str::from_utf8(&self.buf[vstart..vend]).unwrap();

        Some((key, val))
    }
}

#[cfg(test)]
mod tests {
    use crate::zipmap::ZipMap;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use std::{collections::HashMap, time::Instant};
    use std::cmp::max;

    fn rand_usize(m: usize) -> usize {
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        // 简单线性同余生成器（LCG）
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        max(seed % m, 1)
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
    fn test_zipmap_basic() {
        let mut zm = ZipMap::new();

        // 增
        zm.set("a", "1");
        zm.set("b", "22");
        zm.set("c", "333");

        assert_eq!(zm.len(), 3);

        // 查
        assert_eq!(zm.get("a"), Some("1"));
        assert_eq!(zm.get("b"), Some("22"));
        assert_eq!(zm.get("c"), Some("333"));
        assert_eq!(zm.get("d"), None);

        // 改
        zm.set("a", "999");
        assert_eq!(zm.get("a"), Some("999"));
        assert_eq!(zm.get("b"), Some("22"));
        assert_eq!(zm.get("c"), Some("333"));
        assert_eq!(zm.get("d"), None);

        // 删
        assert!(zm.del("b"));
        assert_eq!(zm.len(), 2);
        assert_eq!(zm.get("a"), Some("999"));
        assert_eq!(zm.get("b"), None);
        assert_eq!(zm.get("c"), Some("333"));
        assert_eq!(zm.get("d"), None);

        // 遍历
        zm.rewind();
        while let Some((k, v)) = zm.next() {
            println!("{}: {}", k, v);
        }
    }

    #[test]
    fn test_zipmap_random() {
        let mut zm = ZipMap::new();
        let mut map = HashMap::new();

        const TEST_COUNT: usize = 1000;
        const MAX_KEY_LEN: usize = 2;
        const MAX_VAL_LEN: usize = 10;

        for i in 0..100 {
            for _ in 0..TEST_COUNT {
                let key = random_string(rand_usize(MAX_KEY_LEN));
                let val = random_string(rand_usize(MAX_VAL_LEN));

                // 更新 ZipMap 和 HashMap
                zm.set(&key, &val);
                map.insert(key.clone(), val.clone());
            }

            // 随机删除几个 key
            let keys_to_delete: Vec<_> =
                map.keys().take(rand_usize(MAX_VAL_LEN)).cloned().collect();
            for key in keys_to_delete {
                zm.del(&key);
                map.remove(&key);
            }

            // 最后整个 map 对比
            for (k, v) in &map {
                assert_eq!(zm.get(k).unwrap(), v);
            }
            println!("Round {} passed", i + 1);
        }
    }

    #[test]
    fn test_performance() {
        const TEST_COUNT: usize = 10000;
        const MAX_KEY_LEN: usize = 10;
        const MAX_VAL_LEN: usize = 20;
        const MAX_ELEMS: usize = 10;

        let mut key_lens = Vec::new();
        let mut val_lens = Vec::new();
        let mut elem_counts = Vec::new();

        let mut zm_insert_times = Vec::new();
        let mut zm_get_times = Vec::new();
        let mut zm_del_times = Vec::new();

        let mut hm_insert_times = Vec::new();
        let mut hm_get_times = Vec::new();
        let mut hm_del_times = Vec::new();

        for _ in 0..TEST_COUNT {
            let n = rand_usize(MAX_ELEMS);
            elem_counts.push(n);

            let keys: Vec<String> = (0..n)
                .map(|_| random_string(rand_usize(MAX_KEY_LEN)))
                .collect();
            let vals: Vec<String> = (0..n)
                .map(|_| random_string(rand_usize(MAX_VAL_LEN)))
                .collect();

            key_lens.extend(keys.iter().map(|k| k.len()));
            val_lens.extend(vals.iter().map(|v| v.len()));

            // --- ZipMap 测试 ---
            let mut zm = ZipMap::new();
            let start = Instant::now();
            for i in 0..n {
                zm.set(&keys[i], &vals[i]);
            }
            zm_insert_times.push(start.elapsed());

            let start = Instant::now();
            for i in 0..n {
                zm.get(&keys[i]);
            }
            zm_get_times.push(start.elapsed());

            let start = Instant::now();
            for i in 0..n {
                zm.del(&keys[i]);
            }
            zm_del_times.push(start.elapsed());

            // --- HashMap 测试 ---
            let mut hm = HashMap::new();
            let start = Instant::now();
            for i in 0..n {
                hm.insert(keys[i].clone(), vals[i].clone());
            }
            hm_insert_times.push(start.elapsed());

            let start = Instant::now();
            for i in 0..n {
                hm.get(&keys[i]);
            }
            hm_get_times.push(start.elapsed());

            let start = Instant::now();
            for i in 0..n {
                hm.remove(&keys[i]);
            }
            hm_del_times.push(start.elapsed());
        }

        // 计算统计值
        let key_max = key_lens.iter().max().unwrap();
        let key_min = key_lens.iter().min().unwrap();
        let key_avg = key_lens.iter().sum::<usize>() as f64 / key_lens.len() as f64;

        let val_max = val_lens.iter().max().unwrap();
        let val_min = val_lens.iter().min().unwrap();
        let val_avg = val_lens.iter().sum::<usize>() as f64 / val_lens.len() as f64;

        let count_max = elem_counts.iter().max().unwrap();
        let count_min = elem_counts.iter().min().unwrap();
        let count_avg = elem_counts.iter().sum::<usize>() as f64 / elem_counts.len() as f64;

        let max = |v: &Vec<Duration>| *v.iter().max().unwrap();
        let min = |v: &Vec<Duration>| *v.iter().min().unwrap();
        let avg = |v: &Vec<Duration>| v.iter().sum::<Duration>() / v.len() as u32;

        println!("--- Key/Val length & element count ---");
        println!(
            "Key length  : avg={:.2}, min={}, max={}",
            key_avg, key_min, key_max
        );
        println!(
            "Val length  : avg={:.2}, min={}, max={}",
            val_avg, val_min, val_max
        );
        println!(
            "Element count: avg={:.2}, min={}, max={}",
            count_avg, count_min, count_max
        );

        println!("--- ZipMap performance ---");
        println!(
            "Insert: avg={:?}, min={:?}, max={:?}",
            avg(&zm_insert_times),
            min(&zm_insert_times),
            max(&zm_insert_times)
        );
        println!(
            "Get   : avg={:?}, min={:?}, max={:?}",
            avg(&zm_get_times),
            min(&zm_get_times),
            max(&zm_get_times)
        );
        println!(
            "Del   : avg={:?}, min={:?}, max={:?}",
            avg(&zm_del_times),
            min(&zm_del_times),
            max(&zm_del_times)
        );

        println!("--- HashMap performance ---");
        println!(
            "Insert: avg={:?}, min={:?}, max={:?}",
            avg(&hm_insert_times),
            min(&hm_insert_times),
            max(&hm_insert_times)
        );
        println!(
            "Get   : avg={:?}, min={:?}, max={:?}",
            avg(&hm_get_times),
            min(&hm_get_times),
            max(&hm_get_times)
        );
        println!(
            "Del   : avg={:?}, min={:?}, max={:?}",
            avg(&hm_del_times),
            min(&hm_del_times),
            max(&hm_del_times)
        );
    }
}
