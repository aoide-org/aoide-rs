// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

mod environment;
use std::ops::Deref;

use discro::{Publisher, Ref, Subscriber};

pub use self::environment::{Environment, Handle, WeakHandle};

/// File system utilities
pub mod fs;

/// Collection management
pub mod collection;

/// Settings management
pub mod settings;

/// Track management
pub mod track;

pub type ObservableRef<'a, T> = Ref<'a, T>;

/// Manages the mutable, observable state
#[derive(Debug)]
pub struct Observable<T> {
    publisher: Publisher<T>,
}

impl<T> Observable<T> {
    #[must_use]
    pub fn new(initial_value: T) -> Self {
        let publisher = Publisher::new(initial_value);
        Self { publisher }
    }

    #[must_use]
    pub fn read(&self) -> ObservableRef<'_, T> {
        self.publisher.read()
    }

    #[allow(clippy::must_use_candidate)]
    pub fn modify(&self, modify: impl FnOnce(&mut T) -> bool) -> bool {
        self.publisher.modify(modify)
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> Subscriber<T> {
        self.publisher.subscribe_changed()
    }
}

/// Read-only access to an observable.
pub trait ObservableReader<T> {
    /// Read the current value of the observable.
    ///
    /// Holds a read lock until the returned reference is dropped.
    fn read_observable(&self) -> ObservableRef<'_, T>;
}

impl<T> ObservableReader<T> for Observable<T> {
    fn read_observable(&self) -> ObservableRef<'_, T> {
        self.read()
    }
}

impl<T> ObservableReader<T> for T
where
    T: Deref<Target = Observable<T>>,
{
    fn read_observable(&self) -> ObservableRef<'_, T> {
        self.read()
    }
}
