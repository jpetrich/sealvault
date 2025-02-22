// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import SwiftUI

struct ProfileRow: View {
    @ObservedObject var profile: Profile

    private let cornerRadius: Double = 10
    private let maxDapps = 3

    var body: some View {
        HStack(alignment: .top) {
            ProfileImageRectangle(profile: profile)

            VStack(alignment: .leading) {
                Text(profile.displayName)
                    .font(.headline)

                Text(profile.topDapps)
                    .lineLimit(2)
                    .foregroundStyle(.secondary)
            }

            Spacer(minLength: 0)
        }
        .font(.subheadline)
        .accessibilityElement(children: .combine)
    }
}

#if DEBUG
struct ProfileRow_Previews: PreviewProvider {
    static var previews: some View {
        let model = GlobalModel.buildForPreview()
        return PreviewWrapper(profile: model.activeProfile!)
    }

    struct PreviewWrapper: View {
        @State var profile: Profile
        var body: some View {
            ProfileRow(profile: profile)
        }
    }

}
#endif
