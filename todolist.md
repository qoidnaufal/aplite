# TODO
[X] | 20250104 | rewrite shape & layout, i need to preserve the shape's size etc<br>
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
[_] | ........ | rework `IntoView` & `View` plus storage, i think it's too redundant<br>
[_] | ........ | rework layout traversing especially in shape size<br>
[_] | ........ | improve render performance (CPU side) -> reduce draw calls<br>
[_] | ........ | sdf<br>
[_] | ........ | dynamic (runtime) widget insertion<br>
[_] | ........ | render text, ttf parsing<br>
