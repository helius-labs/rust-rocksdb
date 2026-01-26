// Copyright 2020 Lucjan Suski
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod util;

use pretty_assertions::assert_eq;

use rocksdb::{Options, ReadOptions, SstFileReader, SstFileWriter};

#[test]
fn sst_file_reader_basic() {
    let dir = tempfile::Builder::new()
        .prefix("_rust_rocksdb_sstfilereadertest")
        .tempdir()
        .expect("Failed to create temporary path for file reader.");
    let writer_path = dir.path().join("filewriter");

    // Write an SST file
    {
        let opts = Options::default();
        let mut writer = SstFileWriter::create(&opts);
        writer.open(&writer_path).unwrap();
        writer.put(b"k1", b"v1").unwrap();
        writer.put(b"k2", b"v2").unwrap();
        writer.put(b"k3", b"v3").unwrap();
        writer.finish().unwrap();
    }

    // Read the SST file
    {
        let opts = Options::default();
        let reader = SstFileReader::create(opts);
        reader.open(&writer_path).unwrap();

        let mut iter = reader.iterator(ReadOptions::default());
        iter.seek_to_first();

        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k1".as_ref()));
        assert_eq!(iter.value(), Some(b"v1".as_ref()));

        iter.next();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k2".as_ref()));
        assert_eq!(iter.value(), Some(b"v2".as_ref()));

        iter.next();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k3".as_ref()));
        assert_eq!(iter.value(), Some(b"v3".as_ref()));

        iter.next();
        assert!(!iter.valid());
    }
}

#[test]
fn sst_file_reader_seek() {
    let dir = tempfile::Builder::new()
        .prefix("_rust_rocksdb_sstfilereadertest_seek")
        .tempdir()
        .expect("Failed to create temporary path for file reader.");
    let writer_path = dir.path().join("filewriter");

    // Write an SST file
    {
        let opts = Options::default();
        let mut writer = SstFileWriter::create(&opts);
        writer.open(&writer_path).unwrap();
        writer.put(b"k1", b"v1").unwrap();
        writer.put(b"k3", b"v3").unwrap();
        writer.put(b"k5", b"v5").unwrap();
        writer.finish().unwrap();
    }

    // Read the SST file with seek operations
    {
        let opts = Options::default();
        let reader = SstFileReader::create(opts);
        reader.open(&writer_path).unwrap();

        let mut iter = reader.iterator(ReadOptions::default());

        // Seek to exact key
        iter.seek(b"k3");
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k3".as_ref()));
        assert_eq!(iter.value(), Some(b"v3".as_ref()));

        // Seek to key that doesn't exist (should find next key)
        iter.seek(b"k2");
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k3".as_ref()));

        // Seek to key that doesn't exist (should find next key)
        iter.seek(b"k4");
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k5".as_ref()));

        // Seek past all keys
        iter.seek(b"k9");
        assert!(!iter.valid());
    }
}

#[test]
fn sst_file_reader_seek_for_prev() {
    let dir = tempfile::Builder::new()
        .prefix("_rust_rocksdb_sstfilereadertest_seek_prev")
        .tempdir()
        .expect("Failed to create temporary path for file reader.");
    let writer_path = dir.path().join("filewriter");

    // Write an SST file
    {
        let opts = Options::default();
        let mut writer = SstFileWriter::create(&opts);
        writer.open(&writer_path).unwrap();
        writer.put(b"k1", b"v1").unwrap();
        writer.put(b"k3", b"v3").unwrap();
        writer.put(b"k5", b"v5").unwrap();
        writer.finish().unwrap();
    }

    // Read the SST file with seek_for_prev operations
    {
        let opts = Options::default();
        let reader = SstFileReader::create(opts);
        reader.open(&writer_path).unwrap();

        let mut iter = reader.iterator(ReadOptions::default());

        // Seek for prev to exact key
        iter.seek_for_prev(b"k3");
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k3".as_ref()));
        assert_eq!(iter.value(), Some(b"v3".as_ref()));

        // Seek for prev to key that doesn't exist (should find previous key)
        iter.seek_for_prev(b"k4");
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k3".as_ref()));

        // Seek for prev to key before all keys
        iter.seek_for_prev(b"k0");
        assert!(!iter.valid());
    }
}

#[test]
fn sst_file_reader_reverse_iteration() {
    let dir = tempfile::Builder::new()
        .prefix("_rust_rocksdb_sstfilereadertest_reverse")
        .tempdir()
        .expect("Failed to create temporary path for file reader.");
    let writer_path = dir.path().join("filewriter");

    // Write an SST file
    {
        let opts = Options::default();
        let mut writer = SstFileWriter::create(&opts);
        writer.open(&writer_path).unwrap();
        writer.put(b"k1", b"v1").unwrap();
        writer.put(b"k2", b"v2").unwrap();
        writer.put(b"k3", b"v3").unwrap();
        writer.finish().unwrap();
    }

    // Read the SST file in reverse
    {
        let opts = Options::default();
        let reader = SstFileReader::create(opts);
        reader.open(&writer_path).unwrap();

        let mut iter = reader.iterator(ReadOptions::default());
        iter.seek_to_last();

        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k3".as_ref()));
        assert_eq!(iter.value(), Some(b"v3".as_ref()));

        iter.prev();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k2".as_ref()));
        assert_eq!(iter.value(), Some(b"v2".as_ref()));

        iter.prev();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"k1".as_ref()));
        assert_eq!(iter.value(), Some(b"v1".as_ref()));

        iter.prev();
        assert!(!iter.valid());
    }
}

#[test]
fn sst_file_reader_iterator_trait() {
    let dir = tempfile::Builder::new()
        .prefix("_rust_rocksdb_sstfilereadertest_iter_trait")
        .tempdir()
        .expect("Failed to create temporary path for file reader.");
    let writer_path = dir.path().join("filewriter");

    // Write an SST file
    {
        let opts = Options::default();
        let mut writer = SstFileWriter::create(&opts);
        writer.open(&writer_path).unwrap();
        writer.put(b"k1", b"v1").unwrap();
        writer.put(b"k2", b"v2").unwrap();
        writer.put(b"k3", b"v3").unwrap();
        writer.finish().unwrap();
    }

    // Read the SST file using Iterator trait
    {
        let opts = Options::default();
        let reader = SstFileReader::create(opts);
        reader.open(&writer_path).unwrap();

        let mut iter = reader.iterator(ReadOptions::default());
        iter.seek_to_first();

        let items: Vec<_> = iter.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(&*items[0].0, b"k1");
        assert_eq!(&*items[0].1, b"v1");
        assert_eq!(&*items[1].0, b"k2");
        assert_eq!(&*items[1].1, b"v2");
        assert_eq!(&*items[2].0, b"k3");
        assert_eq!(&*items[2].1, b"v3");
    }
}

#[test]
fn sst_file_reader_item() {
    let dir = tempfile::Builder::new()
        .prefix("_rust_rocksdb_sstfilereadertest_item")
        .tempdir()
        .expect("Failed to create temporary path for file reader.");
    let writer_path = dir.path().join("filewriter");

    // Write an SST file
    {
        let opts = Options::default();
        let mut writer = SstFileWriter::create(&opts);
        writer.open(&writer_path).unwrap();
        writer.put(b"k1", b"v1").unwrap();
        writer.finish().unwrap();
    }

    // Read the SST file using item()
    {
        let opts = Options::default();
        let reader = SstFileReader::create(opts);
        reader.open(&writer_path).unwrap();

        let mut iter = reader.iterator(ReadOptions::default());
        iter.seek_to_first();

        let (key, value) = iter.item().unwrap();
        assert_eq!(key, b"k1");
        assert_eq!(value, b"v1");

        iter.next();
        assert!(iter.item().is_none());
    }
}
