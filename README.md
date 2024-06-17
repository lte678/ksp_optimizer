# KSP Optimizer

_Disclaimer: This is a little joke project, and makes no effort at proper software design :)_

This little tool will search through the effectively infinite configuration space of Kerbal Space Program rockets to find a delta-v optimum (or otherwise).

The script is setup to find an optimal rocket with less than 18t mass, and a TWR of >2.0.

Here is one of the configurations it discovers:

´´´
i=14665, NEW STAGE: FL-T200, FL-T400, FL-T200, LV-T30, FL-T100, FL-T400, RT-5, TD-12, Mk1 Command Pod, FL-T200, LV-T30, FL-T100, FL-T100, FL-T100, FL-T200, FL-T100, FL-T200
DELTA-V: 4667m/s | TWR: 2.105788 | TWR (2. STAGE): 2.7107396

=========== STAGE  0 ==========
        PART MASS: 2.55t
        FUEL MASS: 7.55t
         WET MASS: 10.10t
          DELTA-V: 1490m/s
 THRUST TO WEIGHT: 2.11
 BURNOUT ALTITUDE: 24km
 BURNOUT VELOCITY: 682m/s

=========== STAGE  1 ==========
        PART MASS: 2.71t
        FUEL MASS: 5.00t
         WET MASS: 7.71t
          DELTA-V: 3177m/s
 THRUST TO WEIGHT: 2.71
 BURNOUT ALTITUDE: 131km
 BURNOUT VELOCITY: 3237m/s

FINAL DELTA-V: 4667m/s
´´´