// Copyright 2625 Lucjan Suski
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
//`

use crate::{ffi, ffi_util::to_cpath, Error, Options, ReadOptions};

use libc::{self, c_char, size_t};
use std::ptr::NonNull;
use std::{ffi::CString, marker::PhantomData, path::Path};

/// SstFileReader is used to read sst files that are created by SstFileWriter.
pub struct SstFileReader<'a> {
    pub(crate) inner: *mut ffi::rocksdb_sstfilereader_t,
    // Options are needed to be alive when calling open(),
    // so let's make sure it doesn't get, dropped for the lifetime of SstFileReader.
    phantom: PhantomData<&'a Options>,
}

unsafe impl Send for SstFileReader {}
unsafe impl Sync for SstFileReader {}

impl SstFileReader {
    /// Initializes SstFileReader with given DB options.
    pub fn create(opts: &Options) -> Self {
        Self {
            inner: unsafe { ffi::rocksdb_sstfilereader_create(opts.inner) },
            phantom: PhantomData,
        }
    }

    /// Prepare SstFileReader to read from file located at "file_path".
    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let cpath = to_cpath(&path)?;
        self.open_raw(&cpath)
    }

    fn open_raw(&self, cpath: &CString) -> Result<(), Error> {
        unsafe {
            ffi_try!(ffi::rocksdb_sstfilereader_open(
                self.inner,
                cpath.as_ptr() as *const _
            ));

            Ok(())
        }
    }

    /// Creates a new iterator for the SST file.
    pub fn iterator(&self, options: ReadOptions) -> SstFileReaderIterator {
        unsafe {
            SstFileReaderIterator {
                inner: NonNull::new(ffi::rocksdb_sstfilereader_new_iterator(
                    self.inner,
                    options.inner,
                ))
                .expect("rocksdb_sstfilereader_new_iterator should never return null"),
                options,
                phantom: Default::default(),
            }
        }
    }
}

impl Drop for SstFileReader {
    fn drop(&mut self) {
        unsafe {
            ffi::rocksdb_sstfilereader_destroy(self.inner);
        }
    }
}

pub struct SstFileReaderIterator<'a> {
    pub(crate) inner: NonNull<ffi::rocksdb_iterator_t>,
    options: ReadOptions,
    phantom: PhantomData<&'a SstFileReader>,
}

impl<'a> SstFileReaderIterator<'a> {
    /// Returns `true` if the iterator is valid.
    pub fn valid(&self) -> bool {
        unsafe { ffi::rocksdb_iter_valid(self.inner.as_ptr()) != 0 }
    }

    /// Returns an error if the iterator has encountered an error during operation.
    pub fn status(&self) -> Result<(), Error> {
        unsafe {
            ffi_try!(ffi::rocksdb_iter_get_error(self.inner.as_ptr()));
        }
        Ok(())
    }

    /// Seeks to the first key in the SST file.
    pub fn seek_to_first(&mut self) {
        unsafe {
            ffi::rocksdb_iter_seek_to_first(self.inner.as_ptr());
        }
    }

    /// Seeks to the last key in the SST file.
    pub fn seek_to_last(&mut self) {
        unsafe {
            ffi::rocksdb_iter_seek_to_last(self.inner.as_ptr());
        }
    }

    /// Seeks to the specified key or the first key that lexicographically follows it.
    pub fn seek<K: AsRef<[u8]>>(&mut self, key: K) {
        let key = key.as_ref();
        unsafe {
            ffi::rocksdb_iter_seek(
                self.inner.as_ptr(),
                key.as_ptr() as *const c_char,
                key.len() as size_t,
            );
        }
    }

    /// Seeks to the specified key, or the first key that lexicographically precedes it.
    pub fn seek_for_prev<K: AsRef<[u8]>>(&mut self, key: K) {
        let key = key.as_ref();
        unsafe {
            ffi::rocksdb_iter_seek_for_prev(
                self.inner.as_ptr(),
                key.as_ptr() as *const c_char,
                key.len() as size_t,
            );
        }
    }

    /// Seeks to the next key.
    pub fn next(&mut self) {
        if self.valid() {
            unsafe {
                ffi::rocksdb_iter_next(self.inner.as_ptr());
            }
        }
    }

    /// Seeks to the previous key.
    pub fn prev(&mut self) {
        if self.valid() {
            unsafe {
                ffi::rocksdb_iter_prev(self.inner.as_ptr());
            }
        }
    }

    /// Returns a slice of the current key.
    pub fn key(&self) -> Option<&[u8]> {
        if self.valid() {
            Some(self.key_impl())
        } else {
            None
        }
    }

    /// Returns a slice of the current value.
    pub fn value(&self) -> Option<&[u8]> {
        if self.valid() {
            Some(self.value_impl())
        } else {
            None
        }
    }

    /// Returns a pair with slice of the current key and current value.
    pub fn item(&self) -> Option<(&[u8], &[u8])> {
        if self.valid() {
            Some((self.key_impl(), self.value_impl()))
        } else {
            None
        }
    }

    fn key_impl(&self) -> &[u8] {
        unsafe {
            let mut key_len: size_t = 0;
            let key_len_ptr: *mut size_t = &mut key_len;
            let key_ptr = ffi::rocksdb_iter_key(self.inner.as_ptr(), key_len_ptr);
            std::slice::from_raw_parts(key_ptr as *const u8, key_len)
        }
    }

    fn value_impl(&self) -> &[u8] {
        unsafe {
            let mut val_len: size_t = 0;
            let val_len_ptr: *mut size_t = &mut val_len;
            let val_ptr = ffi::rocksdb_iter_value(self.inner.as_ptr(), val_len_ptr);
            std::slice::from_raw_parts(val_ptr as *const u8, val_len)
        }
    }
}

impl<'a> Iterator for SstFileReaderIterator<'a> {
    type Item = Result<(Box<[u8]>, Box<[u8]>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((key, value)) = self.item() {
            let item = (Box::from(key), Box::from(value));
            self.next();
            Some(Ok(item))
        } else {
            self.status().err().map(Result::Err)
        }
    }
}

impl<'a> Drop for SstFileReaderIterator<'a> {
    fn drop(&mut self) {
        unsafe { ffi::rocksdb_iter_destroy(self.inner.as_ptr()) }
    }
}
