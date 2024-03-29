# Riverbed
Open source Minecraft-inspired game made with [Bevy](https://bevyengine.org/) in Rust.

## Current state
![game_screenshot_fog](https://github.com/Inspirateur/riverbed/assets/22746898/49501ffd-d6f7-4d23-84ab-aef415d7e3be)
*featured on this screenshot is an infinitely generated, editable terrain, and a 4km render distance (equivalent to 256 in Minecraft)*  
Single-player for now but it will be multi-player eventually ...

## Hopes and dreams
Riverbed doesn't aim to be a Minecraft clone - even though it totaly is atm - below are some distinctive aspects planned in the (far) future.

### Push the player to work with the environment
- 🌊 implement flowing rivers without infinite water creation  
  *→ will force the player to divert river flow for irrigation*
- 🌏 implement meaningful biomes, with a real impact on various plant growth  
  *→ the player will need to be smart about cultivation*
- 𓍢ִ໋🌷͙֒ make every plant useful, by including them in a BOTW-inspired cooking system and some craft recipies
- 🚂 make trains the fastest way of transportation (no elytras but gliders are cool)  
  *→ finaly giving players a reason to build the pretty train circuits they always wanted to build*
- 💎 make ores renewable  
  *→ mines will be an asset that the player will want to manage, creating railways, lights, etc.*

### Push the RPG aspect but don't shove it in the player's face
- 👹 generate mobs in special mob structures, spawn very few mob outside of it  
  *→ if the player decides to spend a few hours building something, let them be; they will need to fight for special craft ingredients anyway.*
- ⚔️ make fighting interesting with multiple weapon types, varied enchantments, stamina, dodge and hit mechanics 
- 💰 generate good loot that the player will actually want to find (in mob structures, ruins, etc.)
- 📈 work on a good progression curve, the player shouldn't be able to reach the best metal/mineral in a few minutes

### Quality of life
Big render distance, pretty by default, no cap on items stack, etc.  
There's a plethora of subtle features that makes a game enjoyable, Riverbed shall pay attention to them!

## Development
### Word from the author
Hi, [@Inspirateur](https://github.com/Inspirateur) writing.
Riverbed is a personal project so I plan to stay [BDFL](https://en.wikipedia.org/wiki/Benevolent_dictator_for_life), I have a job to pay for bills 4 days a week, leaving me time to work on the game the remaining days (when I can). 

I'm the primary contributor for now but if you agree with the game vision you're very welcomed to contribute :) !  
(you can contact me on discord at 'inspirateur', and check out the [developer wiki](https://github.com/Inspirateur/riverbed/wiki/Riverbed-Developer-Wiki))

I mean for the project to stay open source and be an inspiration and learning resource for others, and will do my best to keep it well organized, with independant parts of the code published as crates for maximum reusability.

### Thanks
- [@kokounet](https://github.com/kokounet): major contributions on rendering  
*he's a big part of why Riverbed is able to reach 4km of render distance, many thanks!*
- [Denis Périce](https://denis-perice.github.io/): helping me find the ideal water sim  
*it's a work in progress but I'm very glad to have his help*
- [@Involture](https://github.com/Involture): contributed to the efficient packing of chunk data in RAM  
*one of the many invisible optimisations required to make such a game work*

I'm also very grateful for Bevy - the engine powering the game - and the open source Rust ecosystem as a whole, which Riverbed (hopefully) will contribute back to.
