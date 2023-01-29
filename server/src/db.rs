use std::{any::type_name, marker::PhantomData, sync::Arc};

use log::{info, trace};
use serde::{de::DeserializeOwned, Serialize};

use crate::User;

pub struct Db<T: Serialize + DeserializeOwned>(Arc<sled::Db>, PhantomData<T>);

impl<T: Serialize + DeserializeOwned> Clone for Db<T> {
    fn clone(&self) -> Self {
        Db(Arc::clone(&self.0), PhantomData)
    }
}

impl<T: Serialize + DeserializeOwned> Db<T> {
    pub fn open(string: &str) -> Db<T> {
        info!("Opening DB");

        Db(
            Arc::new(sled::open(string).expect("the database to be available")),
            PhantomData,
        )
    }

    pub fn contains(&self, key: &str) -> Result<bool, anyhow::Error> {
        trace!("Checking if {key} exists in {} database", type_name::<T>());

        Ok(self.0.contains_key(key.as_bytes())?)
    }

    pub fn add(&self, key: &str, val: &T) -> Result<(), anyhow::Error> {
        trace!("Adding `{}` to the {} database", &key, type_name::<T>());

        self.0.insert(key.as_bytes(), serde_json::to_vec(val)?)?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<User>, anyhow::Error> {
        trace!("Getting `{}` from the {} database", key, type_name::<T>());

        match self.0.get(key.as_bytes())? {
            Some(v) => Ok(serde_json::from_slice(&v)?),
            None => Ok(None),
        }
    }
}
