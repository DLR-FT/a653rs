use strum::{Display, EnumIter};

#[derive(Debug, Display, EnumIter)]
pub enum HypervisorTraits {
    // P4 Traits
    ApexProcessP4Ext,
    ApexPartitionP4,
    ApexTimeP4Ext,
    ApexErrorP4Ext,
    ApexQueuingPortP4Ext,
    ApexSamplingPortP4Ext,

    // P1 Traits
    ApexProcessP1Ext,
    ApexTimeP1Ext,
    ApexEventP1Ext,
    ApexMutexP1Ext,
    ApexErrorP1Ext,
    ApexBufferP1Ext,
    ApexQueuingPortP1Ext,
    ApexSamplingPortP1Ext,
    ApexSemaphoreP1Ext,
    ApexBlackboardP1Ext,
    // TODO soon P2 Traits
}
