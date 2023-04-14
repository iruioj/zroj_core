import { faker } from "@faker-js/faker";

export type Status =
  | {
      name: "wrong_answer";
    }
  | {
      name: "partial";
      payload: [number, number];
    };

export interface TaskResult {
  status: Status;
  time: number;
  memory: number;
  payload: [string, { str: string; limit: number; truncated: number }][];
}

export interface SubtaskResult {
  status: Status;
  time: number;
  memory: number;
  tasks: TaskResult[];
}

export interface JudgeResult {
  status: Status;
  time: number;
  memory: number;
  detail: {
    Subtask?: SubtaskResult[];
    Tests?: TaskResult[];
  };
}

export function genTestcases() {
  return [];
}
export function genSubtask(): JudgeResult {
  return {
    status: {
      name: "wrong_answer",
    },
    time: 114,
    memory: 514,
    detail: {
      Subtask: [
        {
          status: {
            name: "wrong_answer",
          },
          time: 114,
          memory: 514,
          tasks: [
            {
              status: {
                name: "partial",
                payload: [1.0, 2.0],
              },
              time: 114,
              memory: 514,
              payload: [
                [
                  "stdin",
                  {
                    str: "1 2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "stdout",
                  {
                    str: "2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "answer",
                  {
                    str: "3",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
              ],
            },
            {
              status: {
                name: "partial",
                payload: [1.0, 2.0],
              },
              time: 114,
              memory: 514,
              payload: [
                [
                  "stdin",
                  {
                    str: "1 2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "stdout",
                  {
                    str: "2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "answer",
                  {
                    str: "3",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
              ],
            },
          ],
        },
        {
          status: {
            name: "wrong_answer",
          },
          time: 114,
          memory: 514,
          tasks: [
            {
              status: {
                name: "partial",
                payload: [1.0, 2.0],
              },
              time: 114,
              memory: 514,
              payload: [
                [
                  "stdin",
                  {
                    str: "1 2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "stdout",
                  {
                    str: "2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "answer",
                  {
                    str: "3",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
              ],
            },
            {
              status: {
                name: "partial",
                payload: [1.0, 2.0],
              },
              time: 114,
              memory: 514,
              payload: [
                [
                  "stdin",
                  {
                    str: "1 2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "stdout",
                  {
                    str: "2",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
                [
                  "answer",
                  {
                    str: "3",
                    limit: 1024,
                    truncated: 0,
                  },
                ],
              ],
            },
          ],
        },
      ],
    },
  };
  /* const n_task = faker.datatype.number({ min: 3, max: 8 })
  let records = []
  for (let i = 0; i < n_task; i++) {
    const n_case = faker.datatype.number({ min: 2, max: 10 })
    const case_records = []
    for (let j = 0; j < n_case; j++) {
      case_records.push({
        self: [
          `Test #${j + 1}`,
          [
            'Accepted',
            'Wrong Answer',
            'Time Limit Exceeded'
          ][faker.datatype.number({ min: 0, max: 2 })],
        ]
      })
    }
    records.push({
      self: [`Subtask #${i + 1}`, {
        content: [
          'Accepted',
          'Wrong Answer',
          'Time Limit Exceeded'
        ][faker.datatype.number({ min: 0, max: 2 })],
        span: -1, // -1 表示占据到最后，undefined 和 1 都表示 1
      }],
      children: case_records
    })
  }
  return records */
}

export function genSubmission(seed: number) {
  faker.seed(seed);
  return {
    meta: {
      author: faker.internet.userName(),
      lang: ["C/C++", "Python", "Rust"][
        faker.datatype.number({ min: 0, max: 2 })
      ],
    },
    detail: genSubtask(), // subtask ? genSubtask() : genTestcases()
    // faker.datatype.number({ min: 10, max: 30 }),
  };
}
