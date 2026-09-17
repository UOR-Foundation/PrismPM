module
public import Init
public import PrismPM.Foundation.Holo
public import PrismPM.Foundation.View.Text.V1.Model
public import PrismPM.Production.ControlCoverage
public import PrismPM.Production.ControlCoverageCorpus
public import PrismPM.Production.System
public import PrismPM.Production.SystemValidation
public import PrismPM.Production.SystemValidationCorpus
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace PrismPM.Runtime

@[expose] public def productionRuntimeProfile : Bool := true

end PrismPM.Runtime
