# osrs-nav/webservice
Web API serving pathfinding requests

### Web API
| Route     | Method | Description                                                                       |
|-----------|--------|-----------------------------------------------------------------------------------|
| /path/    | POST   | Path generation request                                                           |
| /varps/   | GET    | Returns the sets of varp/varbit indices whose values are relevant for pathfinding |
| /metrics/ | GET    | Exposes prometheus metrics                                                        |

#### POST /path/
Example body [(2771, 2794, 0)](https://explv.github.io/?centreX=2771&centreY=2794&centreZ=0&zoom=10) -> [(3213, 3427, 0)](https://explv.github.io/?centreX=3213&centreY=3427&centreZ=0&zoom=10)
```json
{
  "start": { "x": 2771, "y": 2794, "plane": 0 },
  "end": { "x": 3213, "y": 3427, "plane": 0 },
  "game_state": {
    "member": true,
    "skill_levels": { "magic": 25 },
    "varps": {
      "273": 110
    },
    "items": {
      "Air rune": 3,
      "Fire rune": 1,
      "Law rune": 1
    }
  }
}
```
Legal Coordinates are in the range `(0, 0, 0)..(6400, 12800, 4)` (exclusive).

Example response
```json
[
    {
        "Edge": {
            "type": "SpellTeleport",
            "spell": "Varrock Teleport"
        }
    },
    {
        "Step": { "x": 3213, "y": 3425, "plane": 0 }
    },
    {
        "Step": { "x": 3212, "y": 3426, "plane": 0 }
    },
    {
        "Step": { "x": 3213, "y": 3427, "plane": 0 }
    }
]
```

If the response code is `200 OK`, the response can be parsed as a JSON Array of [Steps](../pathfinder/src/lib.rs). The array is `null` in case no path could be found.

### Running

```
USAGE:
    webservice --navgrid <NAVGRID>

OPTIONS:
    -h, --help                 Print help information
    -n, --navgrid <NAVGRID>    Path to NavGrid file
```

Use [generator](../generator) to generate a NavGrid file 

Refer to https://rocket.rs/v0.5-rc/guide/configuration/ for documentation on how to configure the server 
