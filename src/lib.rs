mod util;

use itertools::Itertools as _;
use seed::{prelude::*, *};
use web_sys::HtmlInputElement;

use javardry_spoiler::{Class, Item, ItemKind, Monster, Race, Scenario};

#[derive(Debug)]
struct Model {
    plaintext: Option<String>,
    scenario: Option<Scenario>,
    page: Option<Page>,
    refs: Refs,
}

#[derive(Clone, Copy, Debug)]
enum Page {
    Stats,
    Races,
    Classes,
    SpellRealm { id: u32 },
    Items,
    Monsters,
}

#[derive(Debug, Default)]
struct Refs {
    input_file: ElRef<HtmlInputElement>,
}

#[derive(Debug)]
enum Msg {
    InputFileChanged,
    OpenScenario(Vec<u8>),
    PageChanged(Page),
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        plaintext: None,
        scenario: None,
        page: None,
        refs: Refs::default(),
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::InputFileChanged => {
            let files = model.refs.input_file.get().unwrap().files().unwrap();
            let files = gloo_file::FileList::from(files);
            if files.is_empty() {
                return;
            }

            orders.perform_cmd(async move {
                let file = &files[0];
                match gloo_file::futures::read_as_bytes(file).await {
                    Ok(buf) => Some(Msg::OpenScenario(buf)),
                    Err(e) => {
                        log!(format!("cannot read file: {}", e));
                        None
                    }
                }
            });
        }

        Msg::OpenScenario(buf) => {
            let (plaintext, scenario) = match open_scenario(buf) {
                Ok(x) => x,
                Err(e) => {
                    log!(format!("failed to load scenario: {}", e));
                    return;
                }
            };

            model.plaintext = Some(plaintext);
            model.scenario = Some(scenario);
        }

        Msg::PageChanged(page) => {
            model.page = Some(page);
        }
    }
}

fn open_scenario(buf: Vec<u8>) -> anyhow::Result<(String, Scenario)> {
    let plaintext = match String::from_utf8(buf) {
        Ok(x) => x,
        Err(e) => javardry_spoiler::cipher::decrypt(e.into_bytes())?,
    };

    let scenario = Scenario::load_from_plaintext(&plaintext)?;

    Ok((plaintext, scenario))
}

macro_rules! th_fix {
    ($($part:expr),* $(,)?) => {
        th![C!["fixedTable-th"], $($part),*]
    };
}

fn view(model: &Model) -> Node<Msg> {
    div![
        view_form(model),
        IF!(model.scenario.is_some() => view_spoiler(model)),
    ]
}

fn view_form(model: &Model) -> Node<Msg> {
    div![
        attrs! {
            At::Id => "form",
        },
        form![
            label![
                attrs! {
                    At::For => "form-file",
                },
                r#"Open "gameData.dat" or plaintext game data: "#,
            ],
            input![
                el_ref(&model.refs.input_file),
                attrs! {
                    At::Id => "form-file",
                    At::Type => "file",
                },
                ev(Ev::Change, |_| Msg::InputFileChanged),
            ],
            ev(Ev::Submit, |ev| {
                ev.prevent_default();
            }),
        ],
    ]
}

fn view_spoiler(model: &Model) -> Node<Msg> {
    div![
        attrs! {
            At::Id => "spoiler",
        },
        view_spoiler_header(model),
        view_spoiler_menu(model),
        view_spoiler_page(model),
    ]
}

fn view_spoiler_header(model: &Model) -> Node<Msg> {
    let scenario = model.scenario.as_ref().unwrap();

    h2![
        attrs! {
            At::Id => "spoiler-header",
        },
        format!("{} ({})", scenario.title, scenario.id),
    ]
}

fn view_spoiler_menu(model: &Model) -> Node<Msg> {
    let plaintext = model.plaintext.as_ref().unwrap();
    let scenario = model.scenario.as_ref().unwrap();

    let download_url = {
        let blob = gloo_file::Blob::new(plaintext.as_str());
        web_sys::Url::create_object_url_with_blob(blob.as_ref()).unwrap()
    };

    let spell_realm_items: Vec<_> = (0..scenario.spell_realms.len())
        .map(|i| {
            let realm = &scenario.spell_realms[i];
            let label = format!(
                "{}{}",
                realm.name,
                if realm.is_only_for_monster {
                    " (敵専用)"
                } else {
                    ""
                }
            );
            li![view_spoiler_menu_link(
                label,
                Page::SpellRealm { id: realm.id }
            )]
        })
        .collect();

    div![
        attrs! {
            At::Id => "spoiler-menu",
        },
        ul![
            li![view_spoiler_menu_link("特性値", Page::Stats)],
            li![view_spoiler_menu_link("種族", Page::Races)],
            li![view_spoiler_menu_link("職業", Page::Classes)],
            li!["呪文", ul![spell_realm_items]],
            li![view_spoiler_menu_link("アイテム", Page::Items)],
            li![view_spoiler_menu_link("モンスター", Page::Monsters)],
        ],
        div![a![
            attrs! {
                At::Type => "text/plain",
                At::Download => "gameData.txt",
                At::Href => download_url,
            },
            "Download text data",
        ],],
    ]
}

fn view_spoiler_menu_link(label: impl AsRef<str>, page: Page) -> Node<Msg> {
    let label = label.as_ref();

    a![
        attrs! {
            At::Href => "javascript:void(0)",
        },
        label,
        ev(Ev::Click, move |ev| {
            ev.prevent_default();
            Msg::PageChanged(page)
        }),
    ]
}

fn view_spoiler_page(model: &Model) -> Node<Msg> {
    let inner = model.page.map(|page| match page {
        Page::Stats => view_spoiler_page_stats(model),
        Page::Races => view_spoiler_page_races(model),
        Page::Classes => view_spoiler_page_classes(model),
        Page::SpellRealm { id } => view_spoiler_page_spell_realm(model, id),
        Page::Items => view_spoiler_page_items(model),
        Page::Monsters => view_spoiler_page_monsters(model),
    });

    div![
        attrs! {
            At::Id => "spoiler-page",
        },
        inner,
    ]
}

fn view_spoiler_page_stats(model: &Model) -> Node<Msg> {
    let scenario = model.scenario.as_ref().unwrap();

    let rows: Vec<_> = scenario
        .stats
        .iter()
        .map(|stat| {
            tr![
                td![&stat.name],
                td![&stat.name_abbr],
                td![stat.sex_bonus[0].to_string()],
                td![stat.sex_bonus[1].to_string()],
                td![util::bool_str(stat.fixed_on_create)],
                td![util::bool_str(stat.hide)],
            ]
        })
        .collect();

    div![
        h3!["特性値"],
        ul![
            li!["固: キャラ作成時にボーナスポイントを振れない"],
            li!["隠: 隠し特性値"],
        ],
        table![
            thead![tr![
                th!["名前"],
                th!["略称"],
                th!["男"],
                th!["女"],
                th!["固"],
                th!["隠"],
            ]],
            tbody![rows],
        ],
    ]
}

fn view_spoiler_page_races(model: &Model) -> Node<Msg> {
    fn notes(race: &Race) -> Vec<Node<Msg>> {
        let mut nodes = vec![];

        if race.healing != 0 {
            nodes.extend([span![format!("ヒーリング: {}", race.healing)], br![]]);
        }
        if race.spell_cancel != 0 {
            nodes.extend([span![format!("呪文無効化: {}", race.spell_cancel)], br![]]);
        }
        if !race.resist_mask.is_empty() {
            nodes.extend([
                span![format!("抵抗: {}", util::resist_mask_str(race.resist_mask))],
                br![],
            ]);
        }
        if race.cond_to_appear != "true" {
            nodes.extend([span![format!("出現条件: {}", race.cond_to_appear)], br![]]);
        }

        nodes
    }

    let scenario = model.scenario.as_ref().unwrap();

    let header_stats: Vec<_> = scenario
        .stats
        .iter()
        .map(|stat| th![&stat.name_abbr])
        .collect();

    let rows: Vec<_> = scenario
        .races
        .iter()
        .map(|race| {
            let desc = util::strip_text_tags(&race.description);
            let desc = desc.trim();
            let cols_stat: Vec<_> = race.stats.iter().map(|x| td![x.to_string()]).collect();
            tr![
                td![race.id.to_string()],
                td![
                    IF!(!desc.is_empty() => attrs! {
                        At::Title => desc,
                    }),
                    IF!(!desc.is_empty() => style! {
                        St::TextDecoration => "underline",
                        St::TextDecorationStyle => "dotted",
                    }),
                    &race.name,
                ],
                td![&race.name_abbr],
                cols_stat,
                td![race.ac.to_string()],
                td![race.inven_bonus.to_string()],
                td![race.lifetime.to_string()],
                td![notes(race)],
            ]
        })
        .collect();

    div![
        h3!["種族"],
        table![
            thead![tr![
                th!["ID"],
                th!["名前"],
                th!["略称"],
                header_stats,
                th!["AC"],
                th!["所持数"],
                th!["寿命"],
                th!["備考"],
            ]],
            tbody![rows],
        ],
    ]
}

fn view_spoiler_page_classes(model: &Model) -> Node<Msg> {
    fn notes(class: &Class) -> Vec<Node<Msg>> {
        let mut nodes = vec![];

        if !class.attack_debuff_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "打撃効果: {}",
                    util::debuff_mask_str(class.attack_debuff_mask)
                )],
                br![],
            ]);
        }
        if class.cond_to_appear != "true" {
            nodes.extend([span![format!("出現条件: {}", class.cond_to_appear)], br![]]);
        }

        nodes
    }

    let scenario = model.scenario.as_ref().unwrap();

    let header_stats: Vec<_> = scenario
        .stats
        .iter()
        .map(|stat| th_fix![&stat.name_abbr])
        .collect();

    let rows: Vec<_> = scenario
        .classes
        .iter()
        .map(|class| {
            let desc = util::strip_text_tags(&class.description);
            let desc = desc.trim();
            let cols_stat: Vec<_> = class.stats.iter().map(|x| td![x.to_string()]).collect();
            let col_dispell = if let Some(xl) = class.xl_for_dispell {
                td![format!(
                    "LV{}〜 ({})",
                    xl,
                    util::monster_kind_mask_str(class.dispell_mask)
                )]
            } else {
                td![]
            };
            tr![
                td![class.id.to_string()],
                td![
                    IF!(!desc.is_empty() => attrs! {
                        At::Title => desc,
                    }),
                    IF!(!desc.is_empty() => style! {
                        St::TextDecoration => "underline",
                        St::TextDecorationStyle => "dotted",
                    }),
                    &class.name,
                ],
                td![&class.name_abbr],
                td![util::sex_mask_str(class.sex_mask)],
                td![util::alignment_mask_str(class.alignment_mask)],
                cols_stat,
                td![&class.hp_expr],
                td![&class.ac_expr],
                td![&class.hit_expr],
                td![&class.attack_count_expr],
                td![view_dice_triplet(&class.barehand_damage_expr)],
                td![&class.xp_expr],
                col_dispell,
                td![class.thief_skill.to_string()],
                td![util::bool_str(class.can_identify)],
                td![class.inven_bonus.to_string()],
                td![notes(class)],
            ]
        })
        .collect();

    div![
        h3!["職業"],
        div![
            C!["fixedTable-wrapper"],
            table![
                C!["fixedTable-table"],
                thead![tr![
                    th_fix!["ID"],
                    th_fix!["名前"],
                    th_fix!["略称"],
                    th_fix!["性別"],
                    th_fix!["性格"],
                    header_stats,
                    th_fix!["HP"],
                    th_fix!["AC"],
                    th_fix!["命中"],
                    th_fix!["攻撃回数"],
                    th_fix!["素手"],
                    th_fix!["所要経験値"],
                    th_fix!["解呪"],
                    th_fix!["盗賊"],
                    th_fix!["識別"],
                    th_fix!["所持数"],
                    th_fix!["備考"],
                ]],
                tbody![rows],
            ],
        ],
    ]
}

fn view_spoiler_page_spell_realm(model: &Model, realm_id: u32) -> Node<Msg> {
    let scenario = model.scenario.as_ref().unwrap();

    let realm = &scenario.spell_realms[usize::try_from(realm_id).unwrap()];

    let elems_level: Vec<_> = (0..realm.level_count)
        .map(|level| view_spoiler_page_spell_level(model, realm_id, level))
        .collect();

    div![
        h3![format!(
            "呪文 - {}{}",
            realm.name,
            if realm.is_only_for_monster {
                " (敵専用)"
            } else {
                ""
            }
        )],
        elems_level,
    ]
}

fn view_spoiler_page_spell_level(model: &Model, realm_id: u32, level: u32) -> Node<Msg> {
    let scenario = model.scenario.as_ref().unwrap();

    let realm = &scenario.spell_realms[usize::try_from(realm_id).unwrap()];
    let spells = &realm.spells_of_levels[usize::try_from(level).unwrap()];

    let rows: Vec<_> = spells
        .iter()
        .map(|spell| {
            tr![
                td![&spell.name],
                td![spell.cost_mp.to_string()],
                td![util::bool_str(spell.ignore_silence)],
                td![util::bool_str(spell.extra_learn)],
                td![util::strip_text_tags(&spell.description)],
            ]
        })
        .collect();

    div![
        h4![format!("LV {}", level + 1)],
        table![
            thead![tr![
                th!["名前"],
                th!["MP"],
                th!["沈黙無視"],
                th!["特殊習得"],
                th!["解説"],
            ]],
            tbody![rows]
        ],
    ]
}

fn view_spoiler_page_items(model: &Model) -> Node<Msg> {
    fn notes(scenario: &Scenario, item: &Item) -> Vec<Node<Msg>> {
        let curse = item.curse_alignment_mask != 0 || item.curse_sex_mask != 0;
        let curse_always = item.curse_alignment_mask == 0b111 || item.curse_sex_mask == 0b11;

        let mut nodes = vec![];

        if !item.attack_debuff_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "打撃効果: {}",
                    util::debuff_mask_str(item.attack_debuff_mask)
                )],
                br![],
            ]);
        }
        if item.poison_damage != 0 {
            nodes.extend([span![format!("毒: {}", item.poison_damage)], br![]]);
        }
        if !item.slay_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "倍打: {}",
                    util::monster_kind_mask_str(item.slay_mask)
                )],
                br![],
            ]);
        }
        if item.attack_target_count >= 2 {
            nodes.extend([
                span![format!("攻撃対象数: {}", item.attack_target_count)],
                br![],
            ]);
        }

        if item.healing != 0 {
            nodes.extend([span![format!("ヒーリング: {}", item.healing)], br![]]);
        }
        if item.spell_cancel != 0 {
            nodes.extend([span![format!("呪文無効化: {}", item.spell_cancel)], br![]]);
        }
        if !item.resist_mask.is_empty() {
            nodes.extend([
                span![format!("抵抗: {}", util::resist_mask_str(item.resist_mask))],
                br![],
            ]);
        }
        if !item.protect_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "打撃防御: {}",
                    util::monster_kind_mask_str(item.protect_mask)
                )],
                br![],
            ]);
        }

        if item.stats_bonus.iter().any(|&bonus| bonus != 0) {
            let bonus_desc = item
                .stats_bonus
                .iter()
                .enumerate()
                .filter_map(|(i, &bonus)| {
                    (bonus != 0).then(|| format!("{}{:+}", scenario.stats[i].name_abbr, bonus))
                })
                .join(" ");
            nodes.extend([span![format!("修正: {}", bonus_desc)], br![]]);
        }

        if !item.use_str.is_empty() {
            nodes.extend([span![format!("使用: {}", item.use_str)], br![]]);
        }
        if !item.sp_str.is_empty() {
            nodes.extend([span![format!("SP: {}", item.sp_str)], br![]]);
        }

        if let Some(broken_item_id) = item.broken_item_id {
            if (!item.use_str.is_empty() || !item.sp_str.is_empty()) && item.break_prob_expr != "0"
            {
                nodes.extend([
                    span![format!(
                        "壊: {}({}) ({} %)",
                        scenario.items[usize::try_from(broken_item_id).unwrap()].name_ident,
                        broken_item_id,
                        item.break_prob_expr
                    )],
                    br![],
                ]);
            }
        }

        if curse_always {
            nodes.extend([span!["呪い"], br![]]);
        } else if curse {
            let mut ss = vec![];
            if item.curse_alignment_mask != 0 {
                ss.push(util::alignment_mask_str(item.curse_alignment_mask));
            }
            if item.curse_sex_mask != 0 {
                ss.push(util::sex_mask_str(item.curse_sex_mask));
            }
            nodes.extend([span![format!("呪い: {}", ss.join(", "))], br![]]);
        }
        if curse && item.ac != item.ac_curse {
            nodes.extend([span![format!("呪いAC: {}", item.ac_curse)], br![]]);
        }

        if item.hide_in_catalog {
            nodes.extend([span!["図鑑に現れない"], br![]]);
        }

        nodes
    }

    let scenario = model.scenario.as_ref().unwrap();

    let rows: Vec<_> = scenario
        .items
        .iter()
        .map(|item| {
            let desc = util::strip_text_tags(&item.description);
            let desc = desc.trim();
            let col_dice = if matches!(item.kind, ItemKind::Weapon) {
                td![view_dice_triplet(&item.damage_expr)]
            } else {
                td![]
            };
            tr![
                td![item.id.to_string()],
                td![
                    IF!(!desc.is_empty() => attrs! {
                        At::Title => desc,
                    }),
                    IF!(!desc.is_empty() => style! {
                        St::TextDecoration => "underline",
                        St::TextDecorationStyle => "dotted",
                    }),
                    &item.name_ident,
                ],
                td![&item.name_unident],
                td![util::item_kind_str(item.kind)],
                td![util::race_mask_str(scenario, item.equip_race_mask)],
                td![util::class_mask_str(scenario, item.equip_class_mask)],
                td![item.hit_modifier.to_string()],
                td![item.attack_count_modifier.to_string()],
                col_dice,
                td![item.ac.to_string()],
                td![item.ident_difficulty.to_string()],
                td![item.price.to_string()],
                td![item.stock.to_string()],
                td![notes(scenario, item)],
            ]
        })
        .collect();

    div![
        h3!["アイテム"],
        div![
            C!["fixedTable-wrapper"],
            table![
                C!["fixedTable-table"],
                thead![tr![
                    th_fix!["ID"],
                    th_fix!["確定名"],
                    th_fix!["不確定名"],
                    th_fix!["種別"],
                    th_fix!["種族"],
                    th_fix!["職業"],
                    th_fix!["ST"],
                    th_fix!["AT"],
                    th_fix!["ダイス"],
                    th_fix!["AC"],
                    th_fix!["識別"],
                    th_fix!["買値"],
                    th_fix!["在庫"],
                    th_fix!["備考"],
                ]],
                tbody![rows],
            ],
        ],
    ]
}

fn view_spoiler_page_monsters(model: &Model) -> Node<Msg> {
    fn notes(scenario: &Scenario, monster: &Monster) -> Vec<Node<Msg>> {
        let mut nodes = vec![];

        if monster.is_invincible {
            nodes.extend([strong!["無敵"], br![]]);
        }

        if !monster.attack_debuff_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "打撃効果: {}",
                    util::debuff_mask_str(monster.attack_debuff_mask)
                )],
                br![],
            ]);
        }
        if monster.poison_damage != 0 {
            nodes.extend([span![format!("毒: {}", monster.poison_damage)], br![]]);
        }
        if monster.drain_xl != 0 {
            nodes.extend([span![format!("ドレイン: {}", monster.drain_xl)], br![]]);
        }
        if monster.attack_twice {
            nodes.extend([span!["2回攻撃"], br![]]);
        }

        if monster.spell_levels.iter().any(|&level| level != 0) {
            let spell_desc = monster
                .spell_levels
                .iter()
                .enumerate()
                .filter_map(|(i, &level)| {
                    (level != 0).then(|| format!("{}{}", scenario.spell_realms[i].name, level))
                })
                .join(" ");
            nodes.extend([span![format!("呪文: {}", spell_desc)], br![]]);
        }

        if monster.healing != 0 {
            nodes.extend([span![format!("ヒーリング: {}", monster.healing)], br![]]);
        }
        if monster.spell_cancel != 0 {
            nodes.extend([
                span![format!("呪文無効化: {}", monster.spell_cancel)],
                br![],
            ]);
        }
        if !monster.resist_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "抵抗: {}",
                    util::resist_mask_str(monster.resist_mask)
                )],
                br![],
            ]);
        }
        if !monster.vuln_mask.is_empty() {
            nodes.extend([
                span![format!(
                    "弱点: {}",
                    util::resist_mask_str(monster.vuln_mask)
                )],
                br![],
            ]);
        }

        if monster.can_call {
            nodes.extend([span!["仲間を呼ぶ"], br![]]);
        }
        if monster.can_flee {
            nodes.extend([span!["逃走"], br![]]);
        }

        if monster.hide_in_catalog {
            nodes.extend([span!["図鑑に現れない"], br![]]);
        }

        nodes
    }

    let scenario = model.scenario.as_ref().unwrap();

    let header_stats: Vec<_> = scenario
        .stats
        .iter()
        .map(|stat| th_fix![&stat.name_abbr])
        .collect();

    let rows: Vec<_> = scenario
        .monsters
        .iter()
        .map(|monster| {
            let desc = util::strip_text_tags(&monster.description);
            let desc = desc.trim();
            let cols_stat: Vec<_> = monster.stats.iter().map(|x| td![x.to_string()]).collect();
            tr![
                td![monster.id.to_string()],
                td![
                    IF!(!desc.is_empty() => attrs! {
                        At::Title => desc,
                    }),
                    IF!(!desc.is_empty() => style! {
                        St::TextDecoration => "underline",
                        St::TextDecorationStyle => "dotted",
                    }),
                    &monster.name_ident,
                ],
                td![&monster.name_unident],
                td![util::monster_kind_str(monster.kind)],
                td![&monster.xl_expr],
                cols_stat,
                td![&monster.hp_expr],
                td![&monster.ac_expr],
                td![&monster.attack_count_expr],
                td![&monster.damage_expr],
                td![&monster.mp_expr],
                td![&monster.count_in_group_expr],
                td![monster.friendly_prob.to_string()],
                td![notes(scenario, monster)],
            ]
        })
        .collect();

    div![
        h3!["モンスター"],
        div![
            C!["fixedTable-wrapper"],
            table![
                C!["fixedTable-table"],
                thead![tr![
                    th_fix!["ID"],
                    th_fix!["確定名"],
                    th_fix!["不確定名"],
                    th_fix!["種別"],
                    th_fix!["LV"],
                    header_stats,
                    th_fix!["HP"],
                    th_fix!["AC"],
                    th_fix!["AT"],
                    th_fix!["ダイス"],
                    th_fix!["MP"],
                    th_fix!["出現数"],
                    th_fix!["友好"],
                    th_fix!["備考"],
                ]],
                tbody![rows],
            ],
        ],
    ]
}

fn view_dice_triplet(expr: &[impl AsRef<str>]) -> Vec<Node<Msg>> {
    let mut nodes = vec![
        span![expr[0].as_ref()],
        span![
            style! {
                St::Color => "red",
            },
            "d",
        ],
        span![expr[1].as_ref()],
    ];

    if expr[2].as_ref() != "0" {
        nodes.extend([
            span![
                style! {
                    St::Color => "red",
                },
                "+",
            ],
            span![expr[2].as_ref()],
        ]);
    }

    nodes
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
