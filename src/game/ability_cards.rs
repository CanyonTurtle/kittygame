use crate::spritesheet::{Sprite, PresetSprites};


#[derive(PartialEq)]
pub enum AbilityCardTypes {
    Kitty,
    Piggy,
    Lizard,
    Bird
}

pub struct AbilityCard {
    pub card_type: AbilityCardTypes,
    pub sprite: &'static Sprite,
}

pub const N_CARDS: usize = 5;

pub struct AbilityCardStack {
    pub cards: Vec<Option<AbilityCard>>
}

impl AbilityCard {
    pub fn new(card: AbilityCardTypes) -> AbilityCard {
        let preset_sprite_type = match card {
            AbilityCardTypes::Kitty => PresetSprites::KittyCard,
            AbilityCardTypes::Piggy => PresetSprites::PiggyCard,
            AbilityCardTypes::Lizard => PresetSprites::LizardCard,
            AbilityCardTypes::Bird => PresetSprites::BirdCard,
        };
        
        AbilityCard { card_type: card, sprite: Sprite::from_preset(&preset_sprite_type) }
    }
}

pub enum AbilityCardUsageResult {
    NothingHappened,
    GainedTime(u32),
    EnabledFlyForTime(u32),
    // TeleportationEnabled,
}

impl AbilityCardStack {
    pub fn try_push_card(self: &mut Self, card: AbilityCardTypes) {
        if self.cards.len() < N_CARDS {
            self.cards.push(Some(AbilityCard::new(card)));
        }
    }

    pub fn try_use_cards(self: &mut Self) -> AbilityCardUsageResult {
        // if there is a first card, that's the use type.
        if self.cards.is_empty() {
            return AbilityCardUsageResult::NothingHappened;
        }
        match &self.cards[self.cards.len() - 1] {
            None => {},
            Some(card) => {
                let active_card_type = &card.card_type;
                let mut cards_to_consume = [false; N_CARDS];
                cards_to_consume[self.cards.len() - 1] = true;
                let mut n_consumed = 1;
                // consume all adjacent cards of same type
                for (i, other_card) in self.cards[0..self.cards.len() - 1].iter().enumerate().rev() {
                    match other_card {
                        Some(oc) => {
                            if oc.card_type == card.card_type {
                                cards_to_consume[i] = true;
                                n_consumed += 1;
                            } else {
                                break;
                            }
                        },
                        None => {

                        }
                    }
                }

                // apply ability of card
                let abil_card =match &active_card_type {
                    AbilityCardTypes::Kitty => {
                        AbilityCardUsageResult::GainedTime((n_consumed as f32 * (n_consumed as f32 + 1.0) / 2.0) as u32)
                    },
                    AbilityCardTypes::Piggy | AbilityCardTypes::Lizard => AbilityCardUsageResult::GainedTime(10),
                    AbilityCardTypes::Bird => AbilityCardUsageResult::EnabledFlyForTime(0),
                };

                // remove cards off the end (to ensure correct ordering)
                for i in (0..cards_to_consume.len()).rev() {
                    if cards_to_consume[i] {
                        self.cards.remove(i);
                    }
                }
                // self.cards.remove(self.cards.len() - 1);
                return abil_card;

                
            }
        }
        AbilityCardUsageResult::NothingHappened
    }
}