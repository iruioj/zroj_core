import { faker } from '@faker-js/faker'

export function genTestcases() {
  return []
}
export function genSubtask() {
  const n_task = faker.datatype.number({ min: 3, max: 8 })
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
  return records
}

export function genSubmission(seed: number, subtask = true) {
  faker.seed(seed)
  return {
    meta: {
      status: [
        'Accepted',
        'Wrong Answer',
        'Time Limit Exceeded'
      ][faker.datatype.number({ min: 0, max: 2 })],
      author: faker.internet.userName(),
      time: faker.datatype.number({ max: 5000 }),
      memory: faker.datatype.number({ max: 1024 * 1024 * 1024 }),
      lang: [
        'C/C++', 'Python', 'Rust'
      ][faker.datatype.number({ min: 0, max: 2 })]
    },
    detail: subtask ? genSubtask(
    ) : genTestcases(
      // faker.datatype.number({ min: 10, max: 30 }),
    )
  }
}