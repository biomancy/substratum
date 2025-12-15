use crate::lockable::{RegisterReadLocks, RegisterWriteLocks};
use crate::multi_guard::{ReadGuard, WriteGuard};
use crate::{LockId, MultiGuard, MultiLockBuilder, MultiLockError, ReadLockable};
use crate::{ReadLock, Unlock, WriteLock, WriteLockable};
use lock_api::{Mutex, RawMutex, RawRwLock, RwLock};

#[repr(transparent)]
struct RawMutexWrapper<T: RawMutex>(T);

impl<R: RawMutex> RawMutexWrapper<R> {
    pub fn wrap<Data>(mutex: &Mutex<R, Data>) -> &Self {
        // SAFETY: This is safe because RawMutexWrapper has the same representation as T
        // due to #[repr(transparent)].
        unsafe { &*(mutex.raw() as *const R as *const RawMutexWrapper<R>) }
    }

    pub fn id(&self) -> LockId<'_> {
        LockId::from_ptr(self as *const RawMutexWrapper<R>)
    }
}

impl<T: RawMutex> Unlock for RawMutexWrapper<T> {
    unsafe fn unlock(&self) {
        unsafe { RawMutex::unlock(&self.0) };
    }
}

impl<T: RawMutex> WriteLock for RawMutexWrapper<T> {
    fn lock_write(&self) {
        self.0.lock()
    }

    fn try_lock_write(&self) -> bool {
        self.0.try_lock()
    }
}

impl<T: RawMutex> ReadLock for RawMutexWrapper<T> {
    fn lock_read(&self) {
        self.0.lock()
    }

    fn try_lock_read(&self) -> bool {
        self.0.try_lock()
    }
}

impl<R: RawMutex, T> RegisterReadLocks for Mutex<R, T> {
    fn register_read_locks<'a, 'b: 'a>(&'b self, to: &'a mut MultiLockBuilder<'b>) {
        let wrapper = RawMutexWrapper::wrap(self);
        to.add_read_lock(wrapper.id(), wrapper);
    }
}
impl<R: RawMutex, T> ReadLockable for Mutex<R, T> {
    type Data = T;

    fn read_with_guard<'a>(
        &'a self,
        proof: &'a MultiGuard<'a>,
    ) -> Result<ReadGuard<'a, Self::Data>, MultiLockError> {
        let wrapper = RawMutexWrapper::wrap(self);
        proof
            .read_proof(wrapper.id())
            .map(|tk| tk.attach(unsafe { &*self.data_ptr() }))
    }
}

impl<R: RawMutex, T> RegisterWriteLocks for Mutex<R, T> {
    fn register_write_locks<'a, 'b: 'a>(&'b self, to: &'a mut MultiLockBuilder<'b>) {
        let wrapper = RawMutexWrapper::wrap(self);
        to.add_write_lock(wrapper.id(), wrapper);
    }
}

impl<R: RawMutex, T> WriteLockable for Mutex<R, T> {
    type Data = T;

    fn write_with_guard<'a>(
        &'a self,
        proof: &'a MultiGuard<'a>,
    ) -> Result<WriteGuard<'a, Self::Data>, MultiLockError> {
        let wrapper = RawMutexWrapper::wrap(self);
        proof
            .write_proof(wrapper.id())
            .map(|tk| tk.attach(unsafe { &mut *self.data_ptr() }))
    }
}

#[repr(transparent)]
pub struct RawWriteRwLockWrapper<T: RawRwLock>(T);

impl<R: RawRwLock> RawWriteRwLockWrapper<R> {
    pub fn wrap<Data>(rw: &RwLock<R, Data>) -> &Self {
        // SAFETY: This is safe because RawRwLockWrapper has the same representation as T
        // due to #[repr(transparent)].
        unsafe { &*(rw.raw() as *const R as *const RawWriteRwLockWrapper<R>) }
    }

    pub fn id(&self) -> LockId<'_> {
        LockId::from_ptr(self as *const RawWriteRwLockWrapper<R>)
    }
}

impl<T: RawRwLock> Unlock for RawWriteRwLockWrapper<T> {
    unsafe fn unlock(&self) {
        unsafe { RawRwLock::unlock_exclusive(&self.0) };
    }
}

impl<T: RawRwLock> WriteLock for RawWriteRwLockWrapper<T> {
    fn lock_write(&self) {
        self.0.lock_exclusive()
    }

    fn try_lock_write(&self) -> bool {
        self.0.try_lock_exclusive()
    }
}

#[repr(transparent)]
pub struct RawReadRwLockWrapper<T: RawRwLock>(T);

impl<R: RawRwLock> RawReadRwLockWrapper<R> {
    pub fn wrap<Data>(rw: &RwLock<R, Data>) -> &Self {
        // SAFETY: This is safe because RawRwLockWrapper has the same representation as T
        // due to #[repr(transparent)].
        unsafe { &*(rw.raw() as *const R as *const RawReadRwLockWrapper<R>) }
    }

    pub fn id(&self) -> LockId<'_> {
        LockId::from_ptr(self as *const RawReadRwLockWrapper<R>)
    }
}

impl<T: RawRwLock> Unlock for RawReadRwLockWrapper<T> {
    unsafe fn unlock(&self) {
        unsafe { RawRwLock::unlock_shared(&self.0) };
    }
}

impl<T: RawRwLock> ReadLock for RawReadRwLockWrapper<T> {
    fn lock_read(&self) {
        self.0.lock_shared()
    }

    fn try_lock_read(&self) -> bool {
        self.0.try_lock_shared()
    }
}

impl<R: RawRwLock, T> RegisterReadLocks for RwLock<R, T> {
    fn register_read_locks<'a, 'b: 'a>(&'b self, to: &'a mut MultiLockBuilder<'b>) {
        let wrapped = RawReadRwLockWrapper::wrap(self);
        to.add_read_lock(wrapped.id(), wrapped);
    }
}

impl<R: RawRwLock, T> ReadLockable for RwLock<R, T> {
    type Data = T;

    fn read_with_guard<'a>(
        &'a self,
        proof: &'a MultiGuard<'a>,
    ) -> Result<ReadGuard<'a, Self::Data>, MultiLockError> {
        let wrapped = RawReadRwLockWrapper::wrap(self);
        proof
            .read_proof(wrapped.id())
            .map(|tk| tk.attach(unsafe { &*self.data_ptr() }))
    }
}

impl<R: RawRwLock, T> RegisterWriteLocks for RwLock<R, T> {
    fn register_write_locks<'a, 'b: 'a>(&'b self, to: &'a mut MultiLockBuilder<'b>) {
        let wrapper = RawWriteRwLockWrapper::wrap(self);
        to.add_write_lock(wrapper.id(), wrapper);
    }
}

impl<R: RawRwLock, T> WriteLockable for RwLock<R, T> {
    type Data = T;

    fn write_with_guard<'a>(
        &'a self,
        proof: &'a MultiGuard<'a>,
    ) -> Result<WriteGuard<'a, T>, MultiLockError> {
        let wrapper = RawWriteRwLockWrapper::wrap(self);
        proof
            .write_proof(wrapper.id())
            .map(|tk| tk.attach(unsafe { &mut *self.data_ptr() }))
    }
}
