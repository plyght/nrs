import { SimulationNodeDatum, SimulationLinkDatum } from "d3";

// Graph visualization types
export interface Node extends SimulationNodeDatum {
  id: string;
  is_tag: boolean;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  index?: number;
}

export interface Link extends SimulationLinkDatum<Node> {
  source: string | Node;
  target: string | Node;
}

export interface GraphData {
  nodes: Node[];
  links: Link[];
}

// Note types
export interface Note {
  title: string;
  slug: string;
  preview: string;
  tags: string[];
  last_modified: number;
}
