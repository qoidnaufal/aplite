/*
#########################################################
#                                                       #
#                         CORE                          #
#                                                       #
#########################################################
*/

pub trait Reactive: Track + Notify {}

pub trait Track {
    fn track(&self);

    fn untrack(&self) {}
}

pub trait Notify {
    fn notify(&self);
}

impl<T> Reactive for T where T: Track + Notify {}

/*
#########################################################
#                                                       #
#                     READ & WRITE                      #
#                                                       #
#########################################################
*/

pub trait Read: Track {
    type Value: 'static;

    /// read and apply function to the value, and track the underying signal
    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        self.track();
        self.read_untracked(f)
    }

    /// read value without tracking the signal, and apply a function to the value
    fn read_untracked<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R;
}

pub trait Write {
    type Value: 'static;

    /// updating the value without notifying it's subscribers
    fn write_untracked(&self, f: impl FnOnce(&mut Self::Value));
}

/*
#########################################################
#                                                       #
#                     TRACK + READ                      #
#                                                       #
#########################################################
*/

pub trait Get
where
    Self: Read,
    <Self as Read>::Value: Clone,
{
    /// track the signal & clone the value
    fn get(&self) -> <Self as Read>::Value {
        self.track();
        self.get_untracked()
    }

    fn get_untracked(&self) -> <Self as Read>::Value {
        self.read_untracked(|val| val.clone())
    }
}

impl<T> Get for T where T: Track + Read, T::Value: Clone, {}

/*
#########################################################
#                                                       #
#                     NOTIFY + WRITE                    #
#                                                       #
#########################################################
*/

pub trait Set: Notify + Write {
    /// update the value directly and notify the subscribers
    fn set(&self, value: <Self as Write>::Value) {
        self.set_untracked(value);
        self.notify();
    }

    fn set_untracked(&self, value: <Self as Write>::Value) {
        self.write_untracked(|val| *val = value);
    }
}

impl<T: Notify + Write> Set for T {}

pub trait Update: Notify + Write {
    /// update the value via a closure and notify the subscribers
    fn update(&self, f: impl FnOnce(&mut <Self as Write>::Value)) {
        self.write_untracked(f);
        self.notify();
    }

    fn update_untracked(&self, f: impl FnOnce(&mut <Self as Write>::Value)) {
        self.write_untracked(f);
    }
}

impl<T: Notify + Write> Update for T {}

/*
#########################################################
#                                                       #
#                       DISPOSE                         #
#                                                       #
#########################################################
*/

pub trait Dispose {
    /// remove and untrack this signal from the reactive system
    /// be careful accessing the value of disposed signal will cause [`panic!()`](core::panic)
    fn dispose(&self);

    /// check if a signal has been disposed or not
    fn is_disposed(&self) -> bool;
}
