# TODO
[X] | 20250104 | rewrite shape & layout, i need to preserve the shapes size etc<br>
[X] | 20250104 | fix cursor to stick only to the currently clicked object, to avoid hover collision<br>
[X] | 20250106 | function callback using `CallBack(*mut dyn FnMut())`<br>
[X] | 20250106 | layout vertices & indices use `Vec` instead of `HashMap`, because i think it might be slow as fuck when the data becomes too large<br>
[X] | 20250107 | callback using `Box<dyn FnMut(&mut T)>` & signal using `Rc<RefCell>`<br>
[X] | 20250108 | minimum redraw<br>
[X] | 20250112 | texture, bind group<br>
[X] | 20250112 | cpu side transform<br>
[X] | 20250116 | compile time texture collection<br>
[X] | 20250118 | gpu side transform via uniform buffer<br>
[X] | 20250118 | fix texture for each id<br>
[X] | 20250119 | signal injection & handling on each shape<br>
[X] | 20250125 | layouting (start from each widget) & later just scaling on storage<br>
[X] | 20250126 | fixed hover detection to the topmost object<br>
[X] | 20250128 | fixed layouting algorithm<br>
[X] | 20250216 | improve render performance (CPU side) -> use storage buffer<br>
[_] | ........ | back to shape!!!<br>
[_] | ........ | current storage is highly inefficient<br>
[_] | ........ | rework `IntoView` & `View` plus storage, i think its too redundant<br>
[_] | ........ | rework layout traversing especially in shape size<br>
[_] | ........ | improve render performance (CPU side) -> reduce draw calls<br>
[_] | ........ | sdf<br>
[_] | ........ | dynamic (runtime) widget insertion<br>
[_] | ........ | render text, ttf parsing<br>

### Thoughts on Reactivity
There are some reworks need to be done regarding how `Reactivity` should work. First, the way `widgets` doesnt actually exist but `Element` does need to be rethought.
Originally the idea was to avoid "double storage", but this is making it difficult to manage the display state with regards to `Reactivity`.

Second, maybe the `FnOnce(&mut Style)` on each widgets needs to be stored, kinda similar to how `Callback` works? Perhaps this will play well with `Signal`'s `subscriber`.
The question would then be, what's `subscriber`? Is it the `widgets`, or is it the `Signal`s? And how do I manage them?

Third, how should I optimize the relation between `Style` and `Element`? Especially since `Style` also seems need to exist, should I just add `pos` field to it?
