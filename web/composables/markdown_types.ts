export type Node =
  | Root
  | Heading
  | Text
  | Paragraph
  | Code
  | InlineMath
  | TwoColumns;

export type Root = {
  type: "root";
  children: Node[];
};

export type Heading = {
  type: "heading";
  children: Node[];
  depth: number;
};

export type Text = {
  type: "text";
  value: string;
};

export type Paragraph = {
  type: "paragraph";
  children: Node[];
};
export type Code = {
  type: "code";
  value: string;
  lang: string | null;
  meta: string | null;
};

export type InlineMath = {
  type: "inlineMath";
  value: string;
};

export type TwoColumns = {
  type: "twoColumns";
  left: Node;
  right: Node;
};
