#[macro_export]
macro_rules! make_settings_struct {
    ( $vis:vis $name:ident { $( $var:ident: $string_name:tt, $ty:tt, $args:tt ),* , } ) => {
        #[allow(clippy::identity_op)]
        #[allow(unused_parens)]
        $vis struct $name {
            $(pub $var: Setting<$ty> ),*
        }
        impl $name {
            pub fn new() -> Self {
                $(
                    #[allow(non_upper_case_globals)]
                    const $var: Setting<$ty> = {
                        const INFO: $ty = <$ty>::from_tuple($args);
                        Setting {
                            name: $string_name,
                            value: INFO.default,
                            info: &INFO,
                        }
                    }
                );*;
                Self {
                    $( $var ),*
                }

            }
            pub fn iter(&self) -> impl Iterator<Item=SettingKind> {
                vec![
                    $(
                        SettingKind::$ty(&self.$var),
                    )*
                ].into_iter()
            }
            pub fn iter_mut(&mut self) -> impl Iterator<Item=SettingKindMut> {
                vec![
                    $(
                        SettingKindMut::$ty(&mut self.$var),
                    )*
                ].into_iter()
            }
        }
    };
}

pub(crate) use make_settings_struct;
