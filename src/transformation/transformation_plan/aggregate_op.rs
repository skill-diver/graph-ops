use super::{BuiltInOp, TransformationIOT, TransformationOp};
use crate::{
    infra::pi::{
        storage::{Schema, TabularSchema},
        TransformationConnector, GAF,
    },
    transformation::*,
    FeatureValueType, InfraManager, SeResult,
};
use log::info;

#[derive(Debug)]
pub(crate) struct AggregateOp {
    built_in_op: BuiltInOp,
}

impl AggregateOp {
    pub(crate) fn new(args: TransformationArgs, common_args: CommonTransformationArgs) -> Self {
        Self {
            built_in_op: BuiltInOp::new(GAF::AggregateNeighbors, args, common_args),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl TransformationOp for AggregateOp {
    async fn execute(
        &self,
        data_id: DataIdT,
        input: &TransformationIOT,
    ) -> SeResult<TransformationOutputHandler> {
        info!("Executing neighborhood aggregation for data_id: {data_id}");

        // if the execution infra provides builtin support for neighbor aggregation, use the connector directly
        if self
            .get_execution_connector()
            .supports_func(&GAF::AggregateNeighbors)
        {
            self.built_in_op.execute(data_id, input).await
        } else {
            // express neighborhood aggregation in cypher
            assert!(
                self.get_execution_connector().supports_func(&GAF::Cypher),
                "{:?} does not support neighbor aggregation",
                self.get_common_args().infra_id()
            );
            assert_eq!(input.len(), 1, "Expects one input");
            // TODO(tatiana): if input is a source, load data into infra
            assert!(matches!(
                &input[0],
                TransformationOutputHandler::InfraHandler { .. }
            ));
            let args = self.built_in_op.get_args().as_vertex_feature();
            let edge = args.graph_projection.edges.first().unwrap();
            let edge_src_tlabel = &args.graph_projection.vertices.first().unwrap().0;
            let edge_dst_tlabel = &args.graph_projection.vertices.get(1).unwrap().0;
            let target = if edge_src_tlabel.eq(&args.target_vertex_tlabel) {
                "src"
            } else {
                "dst"
            };
            let algorithm_args = args.algorithm.as_aggregate_neighbor();
            let agg_query = format!(
                "MATCH (src:{edge_src_tlabel})-[r:{edge_tlabel}]-{direction}(dst:{edge_dst_tlabel})
                RETURN DISTINCT {target}.{node_primary_key} AS external_id, {agg_properties}",
                edge_tlabel = &edge.tlabel,
                direction = if edge.directed { ">" } else { "" },
                node_primary_key = args.target_vertex_primary_key,
                agg_properties = algorithm_args
                    .properties
                    .iter()
                    .map(|p| format!(
                        "{agg_func}(toFloat(src.{agg_p}))",
                        agg_func = algorithm_args.func.as_cypher_str(),
                        agg_p = p
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            let executor = self.get_execution_connector().get_graph_executor(
                &GAF::Cypher,
                TransformationArgs::new_cypher_args(
                    agg_query,
                    Schema::Tabular(TabularSchema {
                        field_names: args.output_names.clone(),
                        field_types: vec![FeatureValueType::Float; args.output_names.len()],
                        tlabel: Some(args.target_vertex_tlabel.clone()),
                    }),
                ),
                self.get_common_args().source_storage_types().clone(),
                self.get_common_args().sink_storage_type().cloned().unwrap(),
            );
            Ok(executor.execute(input).await.unwrap())
        }
    }

    fn get_common_args(&self) -> &CommonTransformationArgs {
        self.built_in_op.get_common_args()
    }

    fn get_common_args_mut(&mut self) -> &mut CommonTransformationArgs {
        self.built_in_op.get_common_args_mut()
    }

    fn set_execution_connector(&mut self, infra_manager: &InfraManager) {
        self.built_in_op.set_execution_connector(infra_manager)
    }

    fn get_execution_connector(&self) -> &dyn TransformationConnector {
        self.built_in_op.get_execution_connector()
    }
}
