export interface MetamathData {
  in_progress_theorems: InProgressTheorem[];
}

export interface InProgressTheorem {
  name: string;
  text: string;
}
