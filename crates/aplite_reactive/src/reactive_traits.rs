/*
#########################################################
#                                                       #
#                         CORE                          #
#                                                       #
#########################################################
*/

pub trait Reactive {
    fn mark_dirty(&self);

    fn try_update(&self) -> bool;
}

pub trait Track {
    fn track(&self);

    fn untrack(&self) {}
}

pub trait Notify {
    fn notify(&self);
}

// impl<T> Reactive for T where T: Track + Notify {}

/*
#########################################################
#                                                       #
#                     READ & WRITE                      #
#                                                       #
#########################################################
*/

pub trait Read {
    type Value: 'static;

    /// read the value
    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R;

    fn try_read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> Option<R>;
}

pub trait Write {
    type Value: 'static;

    /// updating the value without notifying it's subscribers
    fn write(&self, f: impl FnOnce(&mut Self::Value));
}

/*
#########################################################
#                                                       #
#                     TRACK + READ                      #
#                                                       #
#########################################################
*/

pub trait Get: Track {
    type Value: Clone + 'static;

    /// track the signal & clone the value
    fn get(&self) -> Self::Value {
        self.track();
        self.get_untracked()
    }

    fn try_get(&self) -> Option<Self::Value> {
        self.track();
        self.try_get_untracked()
    }

    fn get_untracked(&self) -> Self::Value;

    fn try_get_untracked(&self) -> Option<Self::Value>;
}

pub trait With: Track {
    type Value: 'static;

    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Value) -> R
    {
        self.track();
        self.with_untracked(f)
    }

    fn try_with<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::Value) -> R
    {
        self.track();
        self.try_with_untracked(f)
    }

    fn with_untracked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Value) -> R;

    fn try_with_untracked<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::Value) -> R;
}

/*
#########################################################
#                                                       #
#                     NOTIFY + WRITE                    #
#                                                       #
#########################################################
*/

pub trait Set: Notify {
    type Value: 'static;

    /// update the value directly and notify the subscribers
    fn set(&self, value: Self::Value) {
        self.set_untracked(value);
        self.notify();
    }

    fn set_untracked(&self, value: Self::Value);
}

pub trait Update: Notify {
    type Value: 'static;

    /// update the value via a closure and notify the subscribers
    fn update(&self, f: impl FnOnce(&mut Self::Value)) {
        self.update_untracked(f);
        self.notify();
    }

    fn update_untracked(&self, f: impl FnOnce(&mut Self::Value));
}

/*
#########################################################
#                                                       #
#                       DISPOSE                         #
#                                                       #
#########################################################
*/

pub trait Dispose {
    /// Remove and untrack this reactive node from the reactive system
    /// # Panic
    /// Accessing the value of disposed reactive node will cause [`panic!()`](core::panic)
    fn dispose(&self);

    /// Check if a reactive node has been disposed or not
    fn is_disposed(&self) -> bool;
}
