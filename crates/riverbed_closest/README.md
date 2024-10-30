# Riverbed Closest
Utilities to pick plants, biomes, etc. depending on terrain parameters such as temperature, humidity, pH, etc.

### Process overview
1. Collections of points or ranges associated with objects are provided by asset files  
*(⚠️ all coordinates are expected to be in \[0; 1\])*
1. Collections can be queried with points in parameter space (a specific temperature + humidity + pH, etc.)  
and will return the closest object as well as a matching score in ]-inf; 1]; 1 meaning exact match, and negative values meaning the object is not "suitable" to the provided conditions

*The crate also include a "print_coverage" utility that samples the parameter space to estimate the coverage% of each object.*

## "Ranges" implementation (`Vec<([Range<f32>; D], E)>`)
Objects are associated to a range for each parameter, for example a cactus can be associated with a temperature range of \[0.7; 1.0\] and so on for humidity, pH, etc.

The matching score will be computed using the worst matching Range for the provided point, values will be negatives if the coordinate is outside the range.  
*this is useful for picking plants that correspond to the terrain conditions*

Query time scales with the number of objects and has been measured as `130 ns` for 16 objects on my machine.  
(`500 μs` to cover the surface of a 62x62 chunk)

## "Points" implementation (`Vec<([f32; D], E)>`)
Objects are associated to a value for each parameter, for example the desert biome can be associated with a temperature of 0.8 and so on for humidity, pH, etc.

The matching score will be computed using the distance from the closest point, but output values with this method will never be negative.  
*this is useful for blending biomes together.*

Query time scales with the number of objects and has been measured as `30 ns` for 16 objects on my machine.  
(`120 μs` to cover the surface of a 62x62 chunk)
