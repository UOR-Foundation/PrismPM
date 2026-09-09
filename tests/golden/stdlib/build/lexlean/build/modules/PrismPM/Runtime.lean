module
public import Init
public import PrismPM.Foundation.Holo
public import PrismPM.Production.System
public import PrismPM.Production.SystemValidation
public import PrismPM.Production.SystemValidationCorpus
set_option autoImplicit false
namespace PrismPM.Runtime

@[expose] public def productionRuntimeProfile : Bool := true

end PrismPM.Runtime
