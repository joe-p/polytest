# Polytest Test Plan
## Test Suites

### Motorcycle

| Name | Description |
| --- | --- |
| [valid vehicle](#valid-vehicle) | Functionality works as expected when the vehicle is valid and not missing any parts |
| [invalid vehicle](#invalid-vehicle) | Errors are thrown when the vehicle has extra parts, which should never happen |
| [broken vehicle](#broken-vehicle) | When the vehicle has parts that need repair, the respective functions should detect the issue |

### Car

| Name | Description |
| --- | --- |
| [valid vehicle](#valid-vehicle) | Functionality works as expected when the vehicle is valid and not missing any parts |
| [invalid vehicle](#invalid-vehicle) | Errors are thrown when the vehicle has extra parts, which should never happen |
| [broken vehicle](#broken-vehicle) | When the vehicle has parts that need repair, the respective functions should detect the issue |

## Test Groups

### Valid Vehicle

| Name | Description |
| --- | --- |
| [check headlights](#check-headlights) | The function for checking headlights should return true if the vehicle has the correct number of headlights |
| [check tires](#check-tires) | The function for checking tires should return true if the vehicle has the correct number of inflated tires |

### Invalid Vehicle

| Name | Description |
| --- | --- |
| [extra headlight throws error](#extra-headlight-throws-error) | Having an extra headlight is invalid and should throw an error |
| [extra tire throws error](#extra-tire-throws-error) | Having an extra tire is invalid and should throw an error |

### Broken Vehicle

| Name | Description |
| --- | --- |
| [broken headlight is caught](#broken-headlight-is-caught) | The function for checking headlights should return false if the vehicle has a broken headlight |
| [flat tire is caught](#flat-tire-is-caught) | The function for checking tires should return false if the vehicle has a flat tire |

## Test Cases

### check headlights

The function for checking headlights should return true if the vehicle has the correct number of headlights

### check tires

The function for checking tires should return true if the vehicle has the correct number of inflated tires

### broken headlight is caught

The function for checking headlights should return false if the vehicle has a broken headlight

### flat tire is caught

The function for checking tires should return false if the vehicle has a flat tire

### extra tire throws error

Having an extra tire is invalid and should throw an error

### extra headlight throws error

Having an extra headlight is invalid and should throw an error
