# KSP Optimizer

_Disclaimer: This is a little joke project, and makes no effort at proper software design :)_

This little tool will search through the effectively infinite configuration space of Kerbal Space Program rockets to find a delta-v optimum (or otherwise).

The script is setup to find an optimal rocket with less than 18t mass, and a TWR of >2.0.

Here is one of the configurations it discovers:

```
i=3821, NEW STAGE: RT-5 LV-T30 FL-T400 FL-T400 TD-12  // LV-T30 FL-T400 FL-T400 FL-T400 FL-T200 FL-T100 Mk1 Command Pod Mk16 Parachute 
DELTA-V: 4541m/s | TWR: 2.0940354 | TWR (2. STAGE): 1.9678526

=========== STAGE  0 ==========
        PART MASS: 2.24t
        FUEL MASS: 5.05t
         WET MASS: 7.29t
          DELTA-V: 828m/s
 THRUST TO WEIGHT: 2.09
 BURNOUT ALTITUDE: 9km
 BURNOUT VELOCITY: 436m/s

=========== STAGE  1 ==========
        PART MASS: 3.13t
        FUEL MASS: 7.50t
         WET MASS: 10.63t
          DELTA-V: 3712m/s
 THRUST TO WEIGHT: 1.97
 BURNOUT ALTITUDE: 156km
 BURNOUT VELOCITY: 3653m/s

============ ROCKET ============
      LAUNCH MASS: 17.92t
          DELTA-V: 4541m/s
       PART COUNT: 13

FINAL DELTA-V: 4541m/s
```

