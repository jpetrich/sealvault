// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Debug;

use generic_array::ArrayLength;

#[cfg(not(target_os = "ios"))]
use crate::encryption::keychains::in_memory_keychain::InMemoryKeychain;
#[cfg(target_os = "ios")]
use crate::encryption::keychains::ios_keychain::IOSKeychain;
use crate::{
    encryption::{key_material::KeyMaterial, KeyName},
    Error,
};

/// Keychain to securely store secrets on the operating system.
/// Injected by the host language as a Uniffi callback interface.
/// More info: https://mozilla.github.io/uniffi-rs/udl/callback_interfaces.html
///
/// The methods takes ownership of the `KeyEncryptionKey` because on iOS, the Core Foundation
/// function (`CFData::from_arc`) that lets us pass the buffer to the iOS Keychain without an
/// internal copy needs to take ownership of the buffer.
pub(super) trait KeychainImpl: Debug + Send + Sync {
    /// Get an item from the local keychain.
    fn get<N: ArrayLength<u8>>(&self, name: KeyName) -> Result<KeyMaterial<N>, Error>;

    /// Delete an item from the local keychain. The operation is idempotent to simplify cleanup
    /// actions.
    fn delete_local(&self, name: KeyName) -> Result<(), Error>;

    /// Put an item on the local (not-synced) keychain. The operation returns an error if a key by
    /// the same name already exists.
    fn put_local<N: ArrayLength<u8>>(
        &self,
        name: KeyName,
        key: KeyMaterial<N>,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct Keychain {
    // `KeychainImpl` is not object safe, so we can't use `Box<dyn KeychainImpl>`
    #[cfg(target_os = "ios")]
    keychain: IOSKeychain,
    #[cfg(not(target_os = "ios"))]
    keychain: InMemoryKeychain,
}

impl Keychain {
    #[cfg(target_os = "ios")]
    pub fn new() -> Self {
        let keychain = IOSKeychain::new();
        Self { keychain }
    }

    #[cfg(not(target_os = "ios"))]
    pub fn new() -> Self {
        let keychain = InMemoryKeychain::new();
        Self { keychain }
    }

    /// Get a symmetric key from the keychain.
    pub(in crate::encryption) fn get<N: ArrayLength<u8>>(
        &self,
        name: KeyName,
    ) -> Result<KeyMaterial<N>, Error> {
        self.keychain.get::<N>(name)
    }

    pub(in crate::encryption) fn delete(&self, name: KeyName) -> Result<(), Error> {
        self.keychain.delete_local(name)
    }

    /// Store a symmetric key on the local keychain encoded that is available when the device is
    /// unlocked. The operation returns an error if a key by the same name already exists.
    pub(in crate::encryption) fn put_local<N: ArrayLength<u8>>(
        &self,
        name: KeyName,
        key: KeyMaterial<N>,
    ) -> Result<(), Error> {
        self.keychain.put_local(name, key)
    }
}

impl Default for Keychain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use generic_array::typenum::U32;

    use super::*;

    #[test]
    fn error_on_duplicate_item() -> Result<()> {
        let keychain = Keychain::new();
        let key_one: KeyMaterial<U32> = KeyMaterial::random()?;
        let key_two: KeyMaterial<U32> = KeyMaterial::random()?;
        keychain.put_local(KeyName::SkKeyEncryptionKey, key_one)?;
        let res = keychain.put_local(KeyName::SkKeyEncryptionKey, key_two);
        assert!(res.is_err());
        Ok(())
    }
}
