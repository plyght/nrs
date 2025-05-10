export type Note = {
  title: string;
  slug: string;
  preview: string;
  tags: string[];
  last_modified: number;
};

export type GraphNode = {
  id: string;
  is_tag: boolean;
};

export type GraphLink = {
  source: string;
  target: string;
};

export type GraphData = {
  nodes: GraphNode[];
  links: GraphLink[];
};
