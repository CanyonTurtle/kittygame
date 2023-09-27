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
    pub cards: [Option<AbilityCard>; N_CARDS],
    pub next_avail_card_i: u8,
}

impl AbilityCard {
    pub fn new(card: AbilityCardTypes) -> AbilityCard {
        let preset_sprite_type = match card {
            AbilityCardTypes::Kitty => PresetSprites::KittyCard,
            AbilityCardTypes::Piggy => PresetSprites::PiggyCard,
            AbilityCardTypes::Lizard => PresetSprites::LizardCard,
            AbilityCardTypes::Bird => PresetSprites::BirdCard,
        };
        
        AbilityCard { card_type: card, sprite: Sprite::from_preset(preset_sprite_type) }
    }
}

impl AbilityCardStack {
    pub fn try_push_card(self: &mut Self, card: AbilityCardTypes) {
        if self.next_avail_card_i < self.cards.len() as u8 {
            self.cards[self.next_avail_card_i as usize] = Some(AbilityCard::new(card))
        }
    }

    pub fn try_use_cards(self: &mut Self) {
        // if there is a first card, that's the use type.
        match &self.cards[0] {
            None => {},
            Some(card) => {
                let active_card_type = &card.card_type;
                let mut cards_to_consume = [false; N_CARDS];
                for (i, other_card) in self.cards[1..].iter().enumerate() {
                    match other_card {
                        Some(oc) => {
                            if oc.card_type == card.card_type {
                                cards_to_consume[i] = true;
                            }
                        },
                        None => {

                        }
                    }
                }
            }
        }
    }
}