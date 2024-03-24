export default interface WorkFlow {
  export_resources: [number, string][];
  source_field_ids: string[];
  owners: string[];
  body: string;
  description: string;
  name: string;
  variant: { Default: [] } | { UserDefined: string };
  tags: Map<string, string>;
}
