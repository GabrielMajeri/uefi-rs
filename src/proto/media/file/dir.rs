use super::{File, FileAttribute, FileInfo, FileMode, FileProtocolInfo, FromUefi};
use crate::{Result, Status};
use core::ffi::c_void;
use core::result;

/// File wrapper for handling directories
///
/// The File abstraction can handle directories, but does so in a very roundabout way. A dedicated
/// abstraction for directory handling that wraps the unintuitive File API is therefore desirable.
pub struct Directory<'a>(File<'a>);

impl<'a> Directory<'a> {
    /// Wrap a File handle into a Directory
    ///
    /// You should have made sure that the file is indeed a directory beforehand, using
    /// `file.get_info<FileInfo>(...)`. We cannot do it for you because this requires an unbounded
    /// amount of memory and we refrain from calling the UEFI allocator implicitly.
    pub unsafe fn from_file(file: File<'a>) -> Self {
        Directory(file)
    }

    /// Try to open a file relative to this directory.
    ///
    /// This simply forwards to the underlying `File::open` implementation
    pub fn open(
        &mut self,
        filename: &str,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Result<File> {
        self.0.open(filename, open_mode, attributes)
    }

    /// Close this directory handle. Same as dropping this structure.
    pub fn close(self) {}

    /// Closes and deletes this directory
    ///
    /// This simply forwards to the underlying `File::delete` implementation
    pub fn delete(self) -> Result<()> {
        self.0.delete()
    }

    /// Read the next directory entry
    ///
    /// Try to read the next directory entry into `buffer`. If the buffer is too small, report the
    /// required buffer size as part of the error. If there are no more directory entries, return
    /// an empty optional.
    ///
    /// You should make sure that the buffer is correctly aligned for storing a FileInfo. If it
    /// isn't, the first few bytes of storage will be skipped to correct the alignment. Please note
    /// that this reduces the effective storage capacity of the buffer.
    ///
    /// # Arguments
    /// * `buffer`  The target buffer of the read operation
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error, the file was deleted,
    ///                                      or the end of the file was reached before the `read()`.
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::BUFFER_TOO_SMALL`   The buffer is too small to hold a directory entry.
    pub fn read_entry(
        &mut self,
        mut buffer: &mut [u8],
    ) -> result::Result<Option<&'a mut FileInfo>, (Status, usize)> {
        // Correct the alignment of the buffer
        buffer = FileInfo::realign_storage(buffer);

        // Read the directory entry into the aligned storage
        self.0.read(buffer).map(|size| {
            if size != 0 {
                unsafe { Some(FileInfo::from_uefi(buffer.as_mut_ptr() as *mut c_void)) }
            } else {
                None
            }
        })
    }

    /// Start over the process of enumerating directory entries
    pub fn reset_entry_readout(&mut self) -> Result<()> {
        self.0.set_position(0)
    }

    /// Queries some information about a directory
    ///
    /// This simply forwards to the underlying `File::get_info` implementation
    pub fn get_info<Info: FileProtocolInfo>(
        &mut self,
        buffer: &mut [u8],
    ) -> result::Result<&mut Info, (Status, usize)> {
        self.0.get_info::<Info>(buffer)
    }

    /// Sets some information about a directory
    ///
    /// This simply forwards to the underlying `File::set_info` implementation
    pub fn set_info<Info: FileProtocolInfo>(&mut self, info: &Info) -> Result<()> {
        self.0.set_info(info)
    }

    /// Flushes all modified data associated with the directory to the device
    ///
    /// This simply forwards to the underlying `File::flush` implementation
    pub fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}
