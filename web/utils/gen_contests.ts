import { faker } from "@faker-js/faker";

export function genContest() {
  const id = faker.datatype.number({ max: 999 });
  return {
    id,
    title: "[NOI 2023赛前集训]" + faker.music.songName(),
    link: "/contest/" + id,
    startTime: faker.datatype.datetime({ min: 1681550861030 }).getTime(),
    duration: faker.datatype.number({ max: 12 * 24 }) * 1000 * 60 * 5,
    participants: faker.datatype.number({ max: 999 }),
    registered: faker.datatype.boolean(),
  };
}
export function genContests(len: number) {
  faker.seed(0);
  const lst = [];
  for (let i = 0; i < len; i++) {
    lst.push(genContest());
  }
  lst.sort((a, b) => a.id - b.id);
  return lst;
}
